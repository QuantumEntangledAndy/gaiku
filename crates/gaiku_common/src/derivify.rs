/// Traits related to derivaties  voxels

/// Signifies that the chunk has a gradient (1st derivaties)
trait gradify<Coord, Value> {
  /// Get the gradient (1st derivative)
  fn get_gradient(&self, x: Coord, y: Coord, z: Coord) -> [Value; 3];
}

/// Signifies that the chunk has a hessian (2nd derivaties)
trait hessiify<Coord, Value> {
  /// Get the hessian (2st derivative)
  fn get_hessian(&self, x: Coord, y: Coord, z: Coord) -> [Value; 9];
}
