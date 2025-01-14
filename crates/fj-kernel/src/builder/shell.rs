use std::array;

use fj_interop::ext::{ArrayExt, SliceExt};
use fj_math::Scalar;
use iter_fixed::IntoIteratorFixed;

use crate::{
    algorithms::transform::TransformObject,
    builder::{
        FaceBuilder, HalfEdgeBuilder, SurfaceBuilder, SurfaceVertexBuilder,
    },
    insert::Insert,
    objects::{Face, FaceSet, HalfEdge, Objects, Shell, SurfaceVertex, Vertex},
    partial::{
        Partial, PartialCurve, PartialCycle, PartialFace, PartialGlobalEdge,
        PartialHalfEdge, PartialObject, PartialSurface, PartialSurfaceVertex,
        PartialVertex,
    },
    services::Service,
    storage::Handle,
};

/// API for building a [`Shell`]
///
/// Also see [`Shell::builder`].
pub struct ShellBuilder {
    /// The faces that make up the [`Shell`]
    pub faces: FaceSet,
}

impl ShellBuilder {
    /// Build the [`Shell`] with the provided faces
    pub fn with_faces(
        mut self,
        faces: impl IntoIterator<Item = Handle<Face>>,
    ) -> Self {
        self.faces.extend(faces);
        self
    }

    /// Create a cube from the length of its edges
    pub fn with_cube_from_edge_length(
        mut self,
        edge_length: impl Into<Scalar>,
        objects: &mut Service<Objects>,
    ) -> Self {
        let edge_length = edge_length.into();

        // Let's define some short-hands. We're going to need them a lot.
        const Z: Scalar = Scalar::ZERO;
        let h = edge_length / 2.;

        let bottom = {
            let surface =
                objects.surfaces.xy_plane().translate([Z, Z, -h], objects);

            PartialFace::default().with_exterior_polygon_from_points(
                surface,
                [[-h, -h], [h, -h], [h, h], [-h, h]],
            )
        };

        let (sides, top_edges) = {
            let surfaces = bottom
                .exterior
                .read()
                .half_edges
                .iter()
                .map(|half_edge| {
                    let [a, b] =
                        half_edge.read().vertices.clone().map(|mut vertex| {
                            vertex
                                .write()
                                .surface_form
                                .write()
                                .infer_global_position()
                        });
                    let c = a + [Z, Z, edge_length];

                    Partial::from_partial(PartialSurface::plane_from_points([
                        a, b, c,
                    ]))
                })
                .collect::<Vec<_>>();

            let bottoms = bottom
                .exterior
                .read()
                .half_edges
                .iter()
                .zip(&surfaces)
                .map(|(half_edge, surface)| {
                    let global_edge = half_edge.read().global_form.clone();

                    let mut half_edge = PartialHalfEdge::default();

                    half_edge.curve().write().global_form =
                        global_edge.read().curve.clone();

                    for (vertex, global_form) in half_edge
                        .vertices
                        .iter_mut()
                        .zip(&global_edge.read().vertices)
                    {
                        vertex.write().surface_form.write().global_form =
                            global_form.clone();
                    }

                    half_edge.global_form = global_edge;

                    half_edge.update_as_line_segment_from_points(
                        surface.clone(),
                        [[Z, Z], [edge_length, Z]],
                    );

                    Partial::from_partial(half_edge)
                })
                .collect::<Vec<_>>();

            let sides_up = bottoms
                .clone()
                .into_iter()
                .zip(&surfaces)
                .map(|(bottom, surface): (Partial<HalfEdge>, _)| {
                    let from_surface = {
                        let [_, from] = &bottom.read().vertices;
                        let from = from.read();
                        from.surface_form.clone()
                    };
                    let to_surface = PartialSurfaceVertex {
                        position: Some(
                            from_surface.read().position.unwrap()
                                + [Z, edge_length],
                        ),
                        surface: surface.clone(),
                        ..Default::default()
                    };

                    let vertices = [
                        PartialVertex {
                            curve: Partial::from_partial(PartialCurve {
                                surface: from_surface.read().surface.clone(),
                                ..Default::default()
                            }),
                            surface_form: from_surface.clone(),
                            ..Default::default()
                        },
                        PartialVertex {
                            curve: Partial::from_partial(PartialCurve {
                                surface: to_surface.surface.clone(),
                                ..Default::default()
                            }),
                            surface_form: Partial::from_partial(to_surface),
                            ..Default::default()
                        },
                    ]
                    .map(Partial::<Vertex>::from_partial);

                    let global_curve = {
                        let [vertex, _] = &vertices;
                        vertex.read().curve.read().global_form.clone()
                    };
                    let global_vertices =
                        vertices.each_ref_ext().map(|vertex| {
                            vertex
                                .read()
                                .surface_form
                                .read()
                                .global_form
                                .clone()
                        });

                    let mut half_edge = PartialHalfEdge {
                        vertices,
                        global_form: Partial::from_partial(PartialGlobalEdge {
                            curve: global_curve,
                            vertices: global_vertices,
                        }),
                    };
                    half_edge.update_as_line_segment();

                    Partial::from_partial(half_edge)
                })
                .collect::<Vec<_>>();

            let sides_down = {
                let mut sides_up_prev = sides_up.clone();
                sides_up_prev.rotate_right(1);

                bottoms
                    .clone()
                    .into_iter()
                    .zip(sides_up_prev)
                    .zip(&surfaces)
                    .map(
                        |((bottom, side_up_prev), surface): (
                            (_, Partial<HalfEdge>),
                            _,
                        )| {
                            let [_, from] =
                                side_up_prev.read().vertices.clone();
                            let [to, _] = bottom.read().vertices.clone();

                            let to = to.read().surface_form.clone();
                            let from = PartialSurfaceVertex {
                                position: Some(
                                    to.read().position.unwrap()
                                        + [Z, edge_length],
                                ),
                                surface: surface.clone(),
                                global_form: from
                                    .read()
                                    .surface_form
                                    .read()
                                    .global_form
                                    .clone(),
                            };

                            let curve = PartialCurve {
                                global_form: side_up_prev
                                    .read()
                                    .curve()
                                    .read()
                                    .global_form
                                    .clone(),
                                ..Default::default()
                            };

                            let vertices = [
                                PartialVertex {
                                    curve: Partial::from_partial(
                                        PartialCurve {
                                            surface: from.surface.clone(),
                                            ..curve.clone()
                                        },
                                    ),
                                    surface_form: Partial::from_partial(from),
                                    ..Default::default()
                                },
                                PartialVertex {
                                    curve: Partial::from_partial(
                                        PartialCurve {
                                            surface: to.read().surface.clone(),
                                            ..curve.clone()
                                        },
                                    ),
                                    surface_form: to.clone(),
                                    ..Default::default()
                                },
                            ]
                            .map(Partial::<Vertex>::from_partial);

                            let global_vertices =
                                vertices.each_ref_ext().map(|vertex| {
                                    vertex
                                        .read()
                                        .surface_form
                                        .read()
                                        .global_form
                                        .clone()
                                });

                            let mut half_edge = PartialHalfEdge {
                                vertices,
                                global_form: Partial::from_partial(
                                    PartialGlobalEdge {
                                        vertices: global_vertices,
                                        curve: curve.global_form,
                                    },
                                ),
                            };
                            half_edge.update_as_line_segment();

                            Partial::from_partial(half_edge)
                        },
                    )
                    .collect::<Vec<_>>()
            };

            let tops = sides_up
                .clone()
                .into_iter()
                .zip(sides_down.clone())
                .map(|(side_up, side_down): (_, Partial<HalfEdge>)| {
                    let [_, from] = side_up.read().vertices.clone();
                    let [to, _] = side_down.read().vertices.clone();

                    let from = from.read().surface_form.clone();
                    let to = to.read().surface_form.clone();

                    let from = PartialVertex {
                        curve: Partial::from_partial(PartialCurve {
                            surface: from.read().surface.clone(),
                            ..Default::default()
                        }),
                        surface_form: from.clone(),
                        ..Default::default()
                    };
                    let to = PartialVertex {
                        curve: Partial::from_partial(PartialCurve {
                            surface: to.read().surface.clone(),
                            ..Default::default()
                        }),
                        surface_form: to.clone(),
                        ..Default::default()
                    };

                    let vertices =
                        [from, to].map(Partial::<Vertex>::from_partial);
                    let global_curve = {
                        let [vertex, _] = &vertices;
                        vertex.read().curve.read().global_form.clone()
                    };
                    let global_vertices =
                        vertices.each_ref_ext().map(|vertex| {
                            vertex
                                .read()
                                .surface_form
                                .read()
                                .global_form
                                .clone()
                        });

                    let mut half_edge = PartialHalfEdge {
                        vertices,
                        global_form: Partial::from_partial(PartialGlobalEdge {
                            curve: global_curve,
                            vertices: global_vertices,
                        }),
                    };
                    half_edge.update_as_line_segment();

                    Partial::from_partial(half_edge)
                })
                .collect::<Vec<_>>();

            let sides = bottoms
                .into_iter()
                .zip(sides_up)
                .zip(tops.clone())
                .zip(sides_down)
                .map(|(((bottom, side_up), top), side_down)| {
                    let mut cycle = PartialCycle::default();
                    cycle.half_edges.extend([bottom, side_up, top, side_down]);

                    PartialFace {
                        exterior: Partial::from_partial(cycle),
                        ..Default::default()
                    }
                })
                .collect::<Vec<_>>();

            (sides, tops)
        };

        let top = {
            let surface =
                objects.surfaces.xy_plane().translate([Z, Z, h], objects);

            let mut top_edges = top_edges;
            top_edges.reverse();

            let surface_vertices = {
                let points = [[-h, -h], [-h, h], [h, h], [h, -h]];

                let mut edges = top_edges.iter();
                let half_edges = array::from_fn(|_| edges.next().unwrap());

                let [a, b, c, d] = points
                    .into_iter_fixed()
                    .zip(half_edges)
                    .collect::<[_; 4]>()
                    .map(|(point, edge)| {
                        let [vertex, _] = edge.read().vertices.clone();
                        let global_vertex = vertex
                            .read()
                            .surface_form
                            .read()
                            .global_form
                            .clone();

                        Partial::from_partial(PartialSurfaceVertex {
                            position: Some(point.into()),
                            surface: Partial::from_full_entry_point(
                                surface.clone(),
                            ),
                            global_form: global_vertex,
                        })
                    });

                [a.clone(), b, c, d, a]
            };

            let mut edges = Vec::new();
            for (surface_vertices, edge) in surface_vertices
                .as_slice()
                .array_windows_ext()
                .zip(top_edges)
            {
                let global_edge = edge.read().global_form.clone();

                let vertices = edge
                    .read()
                    .vertices
                    .each_ref_ext()
                    .into_iter_fixed()
                    .zip(surface_vertices.clone())
                    .collect::<[_; 2]>()
                    .map(
                        |(vertex, surface_form): (
                            _,
                            Partial<SurfaceVertex>,
                        )| PartialVertex {
                            position: vertex.read().position,
                            curve: Partial::from_partial(PartialCurve {
                                surface: surface_form.read().surface.clone(),

                                global_form: vertex
                                    .read()
                                    .curve
                                    .read()
                                    .global_form
                                    .clone(),
                                ..Default::default()
                            }),
                            surface_form: surface_form.clone(),
                        },
                    );

                let mut half_edge = PartialHalfEdge {
                    vertices: vertices.map(Partial::from_partial),
                    global_form: Partial::from_partial(PartialGlobalEdge {
                        curve: global_edge.read().curve.clone(),
                        vertices: global_edge.read().vertices.clone(),
                    }),
                };
                half_edge.update_as_line_segment();

                edges.push(Partial::from_partial(half_edge));
            }

            PartialFace {
                exterior: Partial::from_partial(PartialCycle::new(edges)),
                ..Default::default()
            }
        };

        self.faces.extend(
            [bottom]
                .into_iter()
                .chain(sides)
                .chain([top])
                .map(|face| face.build(objects).insert(objects)),
        );

        self
    }

    /// Build the [`Shell`]
    pub fn build(self, objects: &mut Service<Objects>) -> Handle<Shell> {
        Shell::new(self.faces).insert(objects)
    }
}
