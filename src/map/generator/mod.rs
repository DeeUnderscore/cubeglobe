//! Generators for procedurally generating [`IsoMap`s](struct.IsoMap.html)

mod tergenone;
mod tergentwo;
mod testing;

pub use map::generator::testing::TestingGenerator;
pub use map::generator::tergenone::TerGenOne;
pub use map::generator::tergentwo::TerGenTwo;

use map::IsoMap;

/// A generator capable of returning an
/// [`IsoMap`](map/struct.IsoMap.html).
pub trait Generator {
    fn generate(&self) -> IsoMap;
}
