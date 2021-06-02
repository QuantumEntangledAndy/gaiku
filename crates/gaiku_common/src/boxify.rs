/// Define a component that contains a position.
pub trait Positionable<OriginCoord> {
  fn with_position(position: [OriginCoord; 3]) -> Self;
  fn position(&self) -> [OriginCoord; 3];
}

/// Define a component that contains a 3d size.
pub trait Sizable<Coord> {
  fn with_size(width: Coord, height: Coord, depth: Coord) -> Self;
  fn width(&self) -> Coord;
  fn height(&self) -> Coord;
  fn depth(&self) -> Coord;
}

/// Define a component that is `Sizable` and `Positionable`, also a initializer
pub trait Boxify<OriginCoord, Coord>: Positionable<OriginCoord> + Sizable<Coord> {
  fn new(position: [OriginCoord; 3], width: Coord, height: Coord, depth: Coord) -> Self;
}

/// Signifies that the chunk bounds a volume
trait Boundify<Coord> {
  /// Get the bounds as (min, max)
  fn get_bounds(&self) -> ([Coord; 3], [Coord; 3]);

  /// Gets the size of the bounded domain
  fn get_bounded_size(&self) -> [Coord; 3];
}

trait BoundifyMut<Coord> {
  /// Get the bounds as (min, max)
  fn set_bounds(&mut self, bounds: ([Coord; 3], [Coord; 3]));
}
