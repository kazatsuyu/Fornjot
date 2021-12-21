use crate::{Shape, Shape2d};

/// A 3-dimensional shape
#[derive(Clone, Debug)]
#[repr(C)]
pub enum Shape3d {
    /// The difference of two 3-dimensional shapes
    Difference(Box<Difference>),

    /// A sweep of 2-dimensional shape along the z-axis
    Sweep(Sweep),

    /// A transformed 3-dimensional shape
    Transform(Box<Transform>),

    /// The union of two 3-dimensional shapes
    Union(Box<Union>),
}

impl From<Shape3d> for Shape {
    fn from(shape: Shape3d) -> Self {
        Self::Shape3d(shape.into())
    }
}

/// The difference of two 3-dimensional shapes
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Difference {
    /// The first of the shapes
    pub a: Shape3d,

    /// The second of the shapes
    pub b: Shape3d,
}

impl From<Difference> for Shape {
    fn from(shape: Difference) -> Self {
        Self::Shape3d(Shape3d::Difference(Box::new(shape)))
    }
}

impl From<Difference> for Shape3d {
    fn from(shape: Difference) -> Self {
        Self::Difference(Box::new(shape))
    }
}

/// A transformed 3-dimensional shape
///
/// Transformations are currently limited to a rotation, followed by a
/// translation.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Transform {
    /// The shape being rotated
    pub shape: Shape3d,

    /// The axis of the rotation
    pub axis: [f64; 3],

    /// The angle of the rotation
    pub angle: f64,

    /// The offset of the translation
    pub offset: [f64; 3],
}

impl From<Transform> for Shape {
    fn from(shape: Transform) -> Self {
        Self::Shape3d(Shape3d::Transform(Box::new(shape)))
    }
}

impl From<Transform> for Shape3d {
    fn from(shape: Transform) -> Self {
        Self::Transform(Box::new(shape))
    }
}

/// A sweep of a 2-dimensional shape along the z-axis
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Sweep {
    /// The 2-dimensional shape being swept
    pub shape: Shape2d,

    /// The length of the sweep
    pub length: f64,
}

impl From<Sweep> for Shape {
    fn from(shape: Sweep) -> Self {
        Self::Shape3d(Shape3d::Sweep(shape))
    }
}

impl From<Sweep> for Shape3d {
    fn from(shape: Sweep) -> Self {
        Self::Sweep(shape)
    }
}

/// The union of two 3-dimensional shapes
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Union {
    /// The first of the shapes
    pub a: Shape3d,

    /// The second of the shapes
    pub b: Shape3d,
}

impl From<Union> for Shape {
    fn from(shape: Union) -> Self {
        Self::Shape3d(Shape3d::Union(Box::new(shape)))
    }
}

impl From<Union> for Shape3d {
    fn from(shape: Union) -> Self {
        Self::Union(Box::new(shape))
    }
}
