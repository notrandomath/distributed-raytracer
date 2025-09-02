use std::ops::{Index, IndexMut};
use std::sync::Arc;
use crate::raytracer::prelude::*;
use crate::raytracer::hittable::{Hittable, HitRecord};

#[derive(Serialize, Deserialize)]
pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable>>
}

impl HittableList {
    pub fn new() -> Self {
        HittableList { objects: Vec::new() }
    }

    pub fn new_w_objs(objects: Vec<Arc<dyn Hittable>>) -> Self {
        HittableList { objects }
    }

    pub fn new_w_obj(object: Arc<dyn Hittable>) -> Self {
        let mut list = Self::new();
        list.add(object);
        list
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        self.objects.push(object);
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Arc<dyn Hittable>> {
        self.objects.iter()
    }

    pub fn hits_vec(&self, r: &Ray, ray_t: Interval, _rec: &mut HitRecord) -> Vec<(usize, f64)> {
        let mut temp_rec: HitRecord = HitRecord::default();
        let mut hits: Vec<(usize, f64)> = Vec::new();

        for (idx, object) in self.objects.iter().enumerate() {
            if object.hit(r, Interval::new_min_max(ray_t.min, ray_t.max), &mut temp_rec) {
                hits.push((idx, temp_rec.t));
            }
        }

        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        hits
    }
}

#[typetag::serde]
impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let mut temp_rec: HitRecord = HitRecord::default();
        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;

        for object in &self.objects {
            if object.hit(r, Interval::new_min_max(ray_t.min, closest_so_far), &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec.clone();
            }
        }

        hit_anything
    }
}

impl Index<usize> for HittableList {
    type Output = Arc<dyn Hittable>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.objects[index]
    }
}

impl IndexMut<usize> for HittableList {
    fn index_mut(&mut self, _index: usize) -> &mut Self::Output {
        &mut self.objects[_index]
    }
}