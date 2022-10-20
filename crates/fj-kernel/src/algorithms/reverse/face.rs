use crate::objects::{Face, Objects};

use super::Reverse;

impl Reverse for Face {
    fn reverse(self, objects: &Objects) -> Self {
        let exterior = self.exterior().clone().reverse(objects);
        let interiors =
            self.interiors().map(|cycle| cycle.clone().reverse(objects));

        Face::from_exterior(exterior)
            .with_interiors(interiors)
            .with_color(self.color())
    }
}
