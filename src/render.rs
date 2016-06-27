// pub use primitive::{Sphere};


// pub type Ray = ray::Ray3<float>;
// pub type Point = Point<float>;
use std::cmp;
use Float;
use types::{Point, Vector, Ray, Matrix};
use primitive::Object;
use cgmath::{InnerSpace, SquareMatrix, Transform};

const EPSILON: Float = 0.003;
const MAXVAL: Float = 100.;
// Normalized Vector for diagonally left above

#[derive(Clone)]
pub struct Renderer {
    light_dir: Vector,
    trans: Matrix,
    pub object: Option<Box<Object>>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            light_dir: Vector::new(-2. / 3., 2. / 3., -1. / 3.),
            trans: Matrix::identity(),
            object: None,
        }
    }

    pub fn rotate_from_screen(&mut self, x: f64, y: f64) {
        let euler = ::cgmath::Euler::new(::cgmath::Rad { s: y },
                                         ::cgmath::Rad { s: x },
                                         ::cgmath::Rad { s: 0. });
        let other = Matrix::from(euler);
        self.trans = self.trans.concat(&other);
    }

    pub fn translate_from_screen(&mut self, x: f64, y: f64) {
        let v = Vector::new(-x as Float, y as Float, 0.);
        let other = Matrix::from_translation(v);
        self.trans = self.trans.concat(&other);
    }

    fn cast_ray(&self,
                obj: &Box<Object>,
                r: &Ray,
                light_dir: &Vector,
                origin_value: Float)
                -> (usize, Float) {
        let mut cr = *r;
        let mut value = origin_value;
        let mut iter: usize = 0;

        loop {
            cr.dir = cr.dir.normalize();
            cr.origin = cr.origin + cr.dir * value;
            value = obj.approx_value(cr.origin, EPSILON);
            iter += 1;
            if value > MAXVAL {
                return (iter, 0.);
            }

            if value < EPSILON {
                break;
            }
        }
        let norm = obj.normal(cr.origin);
        let dot = norm.dot(*light_dir);
        if dot < 0. {
            return (iter, 0.);
        }
        return (iter, dot);
    }

    pub fn draw_on_buf(&self, buf: &mut [u8], width: i32, height: i32) {
        let scale = 1. / cmp::min(width, height) as Float;
        let w2 = width / 2;
        let h2 = height / 2;

        let dir_front = self.trans.transform_vector(Vector::new(0., 0., 1.));
        let dir_rl = self.trans.transform_vector(Vector::new(1., 0., 0.));
        let dir_tb = self.trans.transform_vector(Vector::new(0., -1., 0.));
        let light_dir = self.trans.transform_vector(self.light_dir);
        let ray_origin = self.trans.transform_point(Point::new(0., 0., -2.));
        let mut ray = Ray::new(ray_origin, dir_front);

        if let Some(ref my_obj) = self.object {
            let origin_value = my_obj.approx_value(ray.origin, EPSILON);


            let mut index = 0 as usize;
            for y in 0..height {
                let dir_row = dir_front + dir_tb * ((y - h2) as Float * scale);

                for x in 0..width {
                    ray.dir = dir_row + dir_rl * ((x - w2) as Float * scale);

                    let (i, v) = self.cast_ray(my_obj, &ray, &light_dir, origin_value);

                    let b = (255.0 * v * v) as u8;

                    buf[index] = i as u8;
                    index += 1;
                    buf[index] = b;
                    index += 1;
                    buf[index] = b;
                    index += 1;
                    index += 1;
                }
            }
        }
    }
}
