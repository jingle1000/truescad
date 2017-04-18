use super::ast::{Binding, Callable, Expression, ExpressionFn, Value};
use truescad_primitive::{BoundingBox, Intersection, Object, Union};
use truescad_types::{Float, INFINITY, NAN, NEG_INFINITY, Point, Vector};
use std::io::Write;
use std::rc::Rc;


// Macro to create a function binding with multple params.
// The closure that is passed in is expected to take params as a Vec<Value>, with all values in
// the order they were defined in $param => $default.
macro_rules! add_func_multi_param {
    ( $func_name:expr, $closure:expr, $( P $param:expr => $default:expr )*, $vars:expr ) => {
        {
            let mut interface = Vec::new();
            let func_closure: Rc<ExpressionFn> = Rc::new(|env, msg| {
                let mut params = vec![];
                $(
                    if let &Binding::Val(ref value) = env.vars.get($param).unwrap() {
                        params.push(value.clone())
                    } else {
                        panic!("did not find expected param!");
                    }

                )*
                ($closure)(params, &env.objs, msg)
            });
            $(
                interface.push(($param.to_string(), Some(Box::new($default) as Box<Expression>)));
            )*
            $vars.insert($func_name.to_string(),
                                  Binding::Call(Callable {
                                      interface: interface,
                                      ex: func_closure,
                                  }));

        }
    };
}

// Shorthand macro to create a function binding, for closures that only accept a single parameter.
macro_rules! add_func {
    ( $func_name:expr, $closure:expr, $param:expr => $default:expr, $vars:expr ) => {
        add_func_multi_param!($func_name,
            |params: Vec<Value>, subs: &Vec<Box<Object>>, msg: &mut Write| {
                ($closure)(params.get(0).unwrap(), subs, msg)
            }, P $param => $default, $vars);
    };
}

pub fn add_bindings(env: &mut ::std::collections::HashMap<String, Binding>) {
    env.insert("TAU".to_string(),
               Binding::Val(Value::Number(::std::f64::consts::PI * 2.)));
    add_func!("echo",
              |text: &Value, _, msg: &mut Write| {
                  writeln!(msg, "echo: {:?}", text).unwrap();
                  Value::Undef
              },
              "text" => Value::String("".to_string()),
              env);
    add_func!("sphere",
              |r: &Value, _, _| Value::Objects(vec![::truescad_primitive::Sphere::new(r.as_Float())]),
              "r" => Value::Number(1.),
              env);
    add_func!("icylinder",
                        |r: &Value, _, _| Value::Objects(
                            vec![::truescad_primitive::Cylinder::new(r.as_Float())]),
                        "r" => Value::Number(1.),
                        env);
    add_func!("icone",
                                  |r: &Value, _, _| Value::Objects(
                                      vec![::truescad_primitive::Cone::new(r.as_Float(), 0.)]),
                                  "slope" => Value::Number(1.),
                                  env);
    add_func_multi_param!("cube",
              |dim_and_r: Vec<Value>, _,  msg: &mut Write| {
                  if let &Value::Vector(ref dimv) = dim_and_r.get(0).unwrap() {
                      let mut v = Vec::new();
                      for i in 0..3 {
                          v.push(if let Some(&Value::Number(ref x)) = dimv.get(i) {
                              *x as Float
                          } else {
                              writeln!(msg, "invalid dimension value: {:?}, using 0",
                                       dimv.get(i)).unwrap();
                              0.
                          });
                      }
                      return Value::Objects(vec![Intersection::from_vec(vec![
                      ::truescad_primitive::SlabX::new(v[0]),
                      ::truescad_primitive::SlabY::new(v[1]),
                      ::truescad_primitive::SlabZ::new(v[2]) ], dim_and_r.get(1).unwrap().as_Float())
                                                     .unwrap()]);
                  }
                  writeln!(msg, "invalid dimension vector: {:?}, using undef",
                           dim_and_r.get(0)).unwrap();
                  return Value::Undef;
              },
              P "dim" => Value::Vector(vec![Value::Number(1.),
                                          Value::Number(1.),
                                          Value::Number(1.)])
              P "s" => Value::Number(0.),
              env);
    add_func_multi_param!("cylinder",
                        |h_r_r1_r2_s: Vec<Value>, _, msg: &mut Write| {
                            if let &Value::Number(ref h) = h_r_r1_r2_s.get(0).unwrap() {
                                let mut r1 = NAN; let mut r2 = NAN;
                                if let &Value::Number(ref r) = h_r_r1_r2_s.get(1).unwrap() {
                                    r1 = *r as Float; r2 = *r as Float;
                                }
                                if let &Value::Number(ref pr1) = h_r_r1_r2_s.get(2).unwrap() {
                                    r1 = *pr1 as Float;
                                }
                                if let &Value::Number(ref pr2) = h_r_r1_r2_s.get(3).unwrap() {
                                    r2 = *pr2 as Float;
                                }
                                if r1.is_nan() || r2.is_nan() {
                                    writeln!(msg, "invalid radius, returning undef").unwrap();
                                    return Value::Undef;
                                }
                                let mut conie;
                                if r1 == r2 {
                                    conie = ::truescad_primitive::Cylinder::new(r1) as Box<Object>;
                                } else {
                                    let slope = (r2 - r1).abs() / *h as Float;
                                    let offset;
                                    if r1 < r2 {
                                        offset = -r1/ slope - *h as Float * 0.5;
                                    } else {
                                        offset = r2/ slope + *h as Float * 0.5;
                                    }
                                    conie = ::truescad_primitive::Cone::new(slope, offset) as Box<Object>;
                                    let rmax = r1.max(r2);
                                    let conie_box = BoundingBox::new(Point::new(-rmax, -rmax, NEG_INFINITY),
                                                                     Point::new(rmax, rmax, INFINITY));
                                    conie.set_bbox(conie_box);
                                }
                                Value::Objects(vec![Intersection::from_vec(vec![
                                  conie,
                                  ::truescad_primitive::SlabZ::new(*h as Float) ],
                                h_r_r1_r2_s.get(4).unwrap().as_Float()).unwrap()])
                            } else {
                                writeln!(msg, "invalid height, returning undef").unwrap();
                                Value::Undef
                            }
                        },
                        P "h" => Value::Number(1.)
                        P "r" => Value::Number(1.)
                        P "r1" => Value::Undef
                        P "r2" => Value::Undef
                        P "s" => Value::Number(0.),
                        env);
    add_func!("translate",
              |t: &Value, subs: &Vec<Box<Object>>, _| {
                  if subs.len() > 0 {
                      if let &Value::Vector(ref tv) = t {
                          let mut v = Vec::new();
                          for i in 0..3 {
                              v.push(if let Some(x) = tv.get(i) {
                                  x.as_Float_or(0.)
                              } else {
                                  0.
                              });
                          }
                          let union_of_subs = Union::from_vec(subs.clone(), 0.)
                                                  .unwrap();
                          let translated = union_of_subs.translate(Vector::new(v[0],
                                                                                        v[1],
                                                                                        v[2]));
                          return Value::Objects(vec![translated]);
                      }
                  }
                  return Value::Undef;
              },
              "t" => Value::Vector(vec![Value::Number(0.), Value::Number(0.), Value::Number(0.)]),
              env);
    add_func!("rotate",
              |t: &Value, subs: &Vec<Box<Object>>, _| {
                  if subs.len() > 0 {
                      if let &Value::Vector(ref tv) = t {
                          let mut v = Vec::new();
                          for i in 0..3 {
                              v.push(if let Some(x) = tv.get(i) {
                                  x.as_Float_or(0.)
                              } else {
                                  0.
                              });
                          }
                          let union_of_subs = Union::from_vec(subs.clone(), 0.)
                                                  .unwrap();
                          let rotated = union_of_subs.rotate(Vector::new(v[0],
                                                                                  v[1],
                                                                                  v[2]));
                          return Value::Objects(vec![rotated]);
                      }
                  }
                  return Value::Undef;
              },
              "t" => Value::Vector(vec![Value::Number(0.), Value::Number(0.), Value::Number(0.)]),
              env);
    add_func!("scale",
              |t: &Value, subs: &Vec<Box<Object>>, _| {
                  if subs.len() > 0 {
                      if let &Value::Vector(ref tv) = t {
                          let mut v = Vec::new();
                          for i in 0..3 {
                              v.push(if let Some(x) = tv.get(i) {
                                  x.as_Float_or(0.)
                              } else {
                                  0.
                              });
                          }
                          let union_of_subs = Union::from_vec(subs.clone(), 0.)
                                                  .unwrap();
                          let scaled = union_of_subs.scale(Vector::new(v[0], v[1], v[2]));
                          return Value::Objects(vec![scaled]);
                      }
                  }
                  return Value::Undef;
              },
              "t" => Value::Vector(vec![Value::Number(1.), Value::Number(1.), Value::Number(0.)]),
              env);
    add_func!("bend",
                        |w: &Value, subs: &Vec<Box<Object>>, _| {
                            if subs.len() > 0 {
                                if let &Value::Number(ref width) = w {
                                    let union_of_subs = Union::from_vec(
                                        subs.clone(), 0.).unwrap();
                                    let bended = ::truescad_primitive::Bender::new(
                                        union_of_subs, *width as Float);
                                    return Value::Objects(vec![bended]);
                                }
                            }
                            return Value::Undef;
                        },
                        "w" => Value::Number(::std::f64::consts::PI * 2.),
                        env);
    add_func!("twist",
                                            |h: &Value, subs: &Vec<Box<Object>>, _| {
                                                if subs.len() > 0 {
                                                    if let &Value::Number(ref height) = h {
                                                        let union_of_subs = Union::from_vec(
                                                            subs.clone(), 0.).unwrap();
                                                        let twisted = ::truescad_primitive::Twister::new(
                                                            union_of_subs, *height as Float);
                                                        return Value::Objects(vec![twisted]);
                                                    }
                                                }
                                                return Value::Undef;
                                            },
                                            "h" => Value::Number(1.),
                                            env);
    add_func!("union",
              |r: &Value, subs: &Vec<Box<Object>>, _| {
                  if subs.len() > 0 {
                      if let &Value::Number(rf) = r {
                          return Value::Objects(vec![Union::from_vec(subs.clone(),
                                                                                  rf as Float)
                                                         .unwrap()]);
                      }
                  }
                  return Value::Undef;
              },
              "s" => Value::Number(0.),
              env);
    add_func!("intersection",
              |r: &Value, subs: &Vec<Box<Object>>, _| {
                  if subs.len() > 0 {
                      if let &Value::Number(rf) = r {
                          return Value::Objects(
                              vec![Intersection::from_vec(
                                  subs.clone(), rf as Float).unwrap()]);
                      }
                  }
                  return Value::Undef;
              },
              "s" => Value::Number(0.),
              env);
    add_func!("difference",
              |r: &Value, subs: &Vec<Box<Object>>, _| {
                  if subs.len() > 0 {
                      if let &Value::Number(rf) = r {
                          return Value::Objects(
                              vec![Intersection::difference_from_vec(
                                  subs.clone(), rf as Float).unwrap()]);
                      }
                  }
                  return Value::Undef;
              },
              "s" => Value::Number(0.),
              env);
}