//! This module contains utilities to manage the observer.
//!
//! This doc has to be written!!
use crate::transformations;
use crate::transformations::IsHomogeneousMatrix;

// =======================================================================
// CAMERA TRAIT
// =======================================================================

/// Marker trait for Camera classes
pub trait Camera{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32);

    // todo
    //fn fire_ray();
}

// =======================================================================
// ORTHOGONAL CAMERA
// =======================================================================
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OrthogonalCamera<T : IsHomogeneousMatrix>{
    pub transformation : T,
    pub aspect_ratio : f32,
}

impl<T : IsHomogeneousMatrix> OrthogonalCamera<T>{
    pub fn new(transformation : T) -> OrthogonalCamera<T>{
        OrthogonalCamera{transformation, aspect_ratio : 1.0}
    }
}

impl<T : IsHomogeneousMatrix> Camera for OrthogonalCamera<T>{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32){
        self.aspect_ratio = aspect_ratio;
    }
}

// =======================================================================
// PERSPECTIVE CAMERA
// =======================================================================

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PerspectiveCamera<T : IsHomogeneousMatrix>{
    pub transformation : T,
    pub aspect_ratio : f32,
}

impl<T : IsHomogeneousMatrix> PerspectiveCamera<T>{
    pub fn new(transformation : T) -> PerspectiveCamera<T>{
        PerspectiveCamera{transformation, aspect_ratio : 1.0}
    }
}

impl<T : IsHomogeneousMatrix> Camera for PerspectiveCamera<T>{
    fn set_aspect_ratio(&mut self, aspect_ratio: f32){
        self.aspect_ratio = aspect_ratio
    }
}

