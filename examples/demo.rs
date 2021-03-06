#![allow(dead_code)]

use type_utils::TypeUtils;

#[derive(TypeUtils)]
#[pick(pub(self) S1 {pub a, pub(self) b})]
#[tu_derive(Copy, Clone)]
#[omit(S2 {a})]
struct S {
  a: i32,
  b: i32,
  c: i32,
}

#[derive(TypeUtils)]
#[pick(E1 {A, B})]
#[omit(E2 {A})]
enum E {
  A,
  B,
  C,
}

fn main() {
  let _ = S1 { a: 42, b: 42 };
  let s = S2 { b: 42, c: 42 };
  let s_ = s;
  assert!(s.b == s_.b && s.c == s_.c);

  match E1::A {
    E1::A => (),
    E1::B => (),
  }
  match E2::B {
    E2::B => (),
    E2::C => (),
  }
}
