use glam::{Mat4, Vec3};

/// An axis-aligned bounding box.
///
/// The empty box is inverted - `min` above `max` - so accumulation can start
/// from [`Aabb::EMPTY`] and union points or boxes in without a special case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// A box enclosing nothing, ready to be grown by unions.
    pub const EMPTY: Self = Self {
        min: Vec3::splat(f32::INFINITY),
        max: Vec3::splat(f32::NEG_INFINITY),
    };

    /// Smallest box containing every point; [`Aabb::EMPTY`] when there are none.
    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Self {
        let mut aabb = Self::EMPTY;
        for point in points {
            aabb.union_point(point);
        }
        aabb
    }

    /// Grows the box to include `point`.
    pub fn union_point(&mut self, point: Vec3) -> &mut Self {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
        self
    }

    /// Grows the box to include all of `other`.
    pub fn union(&mut self, other: &Self) -> &mut Self {
        self.union_point(other.min);
        self.union_point(other.max);
        self
    }

    /// Size along each axis; negative components when the box is empty.
    pub fn extents(&self) -> Vec3 {
        self.max - self.min
    }

    /// True when the box encloses a non-zero volume on every axis. A box with
    /// no volume at all - a point or a plane - counts as empty here, which is
    /// what an overlap test wants.
    pub fn has_volume(&self) -> bool {
        self.extents().min_element() > 0.0
    }

    /// True when the box encloses nothing, including [`Aabb::EMPTY`].
    pub fn is_empty(&self) -> bool {
        !self.has_volume()
    }

    /// True when both corners are finite, so the box can be projected.
    pub fn is_finite(&self) -> bool {
        self.min.is_finite() && self.max.is_finite()
    }

    /// The eight corners. Bit 0 picks x, bit 1 y and bit 2 z between `min` and
    /// `max`, so the order is stable if the axes ever need to be enumerated.
    pub fn corners(&self) -> [Vec3; 8] {
        std::array::from_fn(|i| {
            Vec3::new(
                if i & 1 == 0 { self.min.x } else { self.max.x },
                if i & 2 == 0 { self.min.y } else { self.max.y },
                if i & 4 == 0 { self.min.z } else { self.max.z },
            )
        })
    }

    /// Overlap of two boxes; empty when they do not overlap.
    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn radius(&self) -> f32 {
        self.extents().length() * 0.5
    }

    /// Bounds of this box's corners after an affine transform.
    pub fn transform_affine(&self, matrix: Mat4) -> Self {
        // All eight corners are transformed because a rotation means the local box
        // is generally not axis-aligned. 
        // IMPROVEMENT: We could just transform an additional vector that points to the midpoint
        // on each axis, and then use the transformed midpoint to find the new min and max. 
        // This would reduce the number of transformations from 8 to 4.
        Self::from_points(self.corners().map(|corner| matrix.transform_point3(corner)))
    }
}
