// This snapshot tests accessing various containers, dereferencing pointers.

struct GlobalConst {
    a: u32,
    b: vec3<u32>,
    c: i32,
}
// tests msl padding insertion for global constants
var<private> msl_padding_global_const: GlobalConst = GlobalConst(0u, vec3<u32>(0u, 0u, 0u), 0);

struct AlignedWrapper {
    @align(8) value: i32
}

struct Bar {
    _matrix: mat4x3<f32>,
    matrix_array: array<mat2x2<f32>, 2>,
    atom: atomic<i32>,
    atom_arr: array<atomic<i32>, 10>,
    arr: array<vec2<u32>, 2>,
    data: array<AlignedWrapper>,
}

@group(0) @binding(0)
var<storage,read_write> bar: Bar;

struct Baz {
    m: mat3x2<f32>,
}

@group(0) @binding(1)
var<uniform> baz: Baz;

@group(0) @binding(2)
var<storage,read_write> qux: vec2<i32>;

fn test_matrix_within_struct_accesses() {
    var idx = 1;

    idx--;

    // loads
    let l0 = baz.m;
    let l1 = baz.m[0];
    let l2 = baz.m[idx];
    let l3 = baz.m[0][1];
    let l4 = baz.m[0][idx];
    let l5 = baz.m[idx][1];
    let l6 = baz.m[idx][idx];

    var t = Baz(mat3x2<f32>(vec2<f32>(1.0), vec2<f32>(2.0), vec2<f32>(3.0)));

    idx++;

    // stores
    t.m = mat3x2<f32>(vec2<f32>(6.0), vec2<f32>(5.0), vec2<f32>(4.0));
    t.m[0] = vec2<f32>(9.0);
    t.m[idx] = vec2<f32>(90.0);
    t.m[0][1] = 10.0;
    t.m[0][idx] = 20.0;
    t.m[idx][1] = 30.0;
    t.m[idx][idx] = 40.0;
}

struct MatCx2InArray {
    am: array<mat4x2<f32>, 2>,
}

@group(0) @binding(3)
var<uniform> nested_mat_cx2: MatCx2InArray;

fn test_matrix_within_array_within_struct_accesses() {
    var idx = 1;

    idx--;

    // loads
    let l0 = nested_mat_cx2.am;
    let l1 = nested_mat_cx2.am[0];
    let l2 = nested_mat_cx2.am[0][0];
    let l3 = nested_mat_cx2.am[0][idx];
    let l4 = nested_mat_cx2.am[0][0][1];
    let l5 = nested_mat_cx2.am[0][0][idx];
    let l6 = nested_mat_cx2.am[0][idx][1];
    let l7 = nested_mat_cx2.am[0][idx][idx];

    var t = MatCx2InArray(array<mat4x2<f32>, 2>());

    idx++;

    // stores
    t.am = array<mat4x2<f32>, 2>();
    t.am[0] = mat4x2<f32>(vec2<f32>(8.0), vec2<f32>(7.0), vec2<f32>(6.0), vec2<f32>(5.0));
    t.am[0][0] = vec2<f32>(9.0);
    t.am[0][idx] = vec2<f32>(90.0);
    t.am[0][0][1] = 10.0;
    t.am[0][0][idx] = 20.0;
    t.am[0][idx][1] = 30.0;
    t.am[0][idx][idx] = 40.0;
}

fn read_from_private(foo: ptr<function, f32>) -> f32 {
    return *foo;
}

fn test_arr_as_arg(a: array<array<f32, 10>, 5>) -> f32 {
    return a[4][9];
}

@vertex
fn foo_vert(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    var foo: f32 = 0.0;
    // We should check that backed doesn't skip this expression
    let baz: f32 = foo;
    foo = 1.0;

    _ = msl_padding_global_const;
    test_matrix_within_struct_accesses();
    test_matrix_within_array_within_struct_accesses();

    // test storage loads
    let _matrix = bar._matrix;
    let arr = bar.arr;
    let index = 3u;
    let b = bar._matrix[index].x;
    let a = bar.data[arrayLength(&bar.data) - 2u].value;
    let c = qux;

    // test pointer types
    let data_pointer: ptr<storage, i32, read_write> = &bar.data[0].value;
    let foo_value = read_from_private(&foo);

    // test array indexing
    var c2 = array<i32, 5>(a, i32(b), 3, 4, 5);
    c2[vi + 1u] = 42;
    let value = c2[vi];

    test_arr_as_arg(array<array<f32, 10>, 5>());

    return vec4<f32>(_matrix * vec4<f32>(vec4<i32>(value)), 2.0);
}

@fragment
fn foo_frag() -> @location(0) vec4<f32> {
    // test storage stores
    bar._matrix[1].z = 1.0;
    bar._matrix = mat4x3<f32>(vec3<f32>(0.0), vec3<f32>(1.0), vec3<f32>(2.0), vec3<f32>(3.0));
    bar.arr = array<vec2<u32>, 2>(vec2<u32>(0u), vec2<u32>(1u));
    bar.data[1].value = 1;
    qux = vec2<i32>();

    return vec4<f32>(0.0);
}

fn assign_through_ptr_fn(p: ptr<function, u32>) {
    *p = 42u;
}

fn assign_array_through_ptr_fn(foo: ptr<function, array<vec4<f32>, 2>>) {
    *foo = array<vec4<f32>, 2>(vec4(1.0), vec4(2.0));
}

fn assign_through_ptr() {
    var val = 33u;
    assign_through_ptr_fn(&val);

    var arr = array<vec4<f32>, 2>(vec4(6.0), vec4(7.0));
    assign_array_through_ptr_fn(&arr);
}

struct AssignToMember {
  x: u32,
}

fn fetch_arg_ptr_member(p: ptr<function, AssignToMember>) -> u32 {
  return (*p).x;
}

fn assign_to_arg_ptr_member(p: ptr<function, AssignToMember>) {
  (*p).x = 10u;
}

fn fetch_arg_ptr_array_element(p: ptr<function, array<u32, 4>>) -> u32 {
  return (*p)[1];
}

fn assign_to_arg_ptr_array_element(p: ptr<function, array<u32, 4>>) {
  (*p)[1] = 10u;
}

fn assign_to_ptr_components() {
   var s1: AssignToMember;
   assign_to_arg_ptr_member(&s1);
   fetch_arg_ptr_member(&s1);

   var a1: array<u32, 4>;
   assign_to_arg_ptr_array_element(&a1);
   fetch_arg_ptr_array_element(&a1);
}

fn index_ptr(value: bool) -> bool {
    var a = array<bool, 1>(value);
    let p = &a;
    return p[0];
}

struct S { m: i32 };

fn member_ptr() -> i32 {
    var s: S = S(42);
    let p = &s;
    return p.m;
}

struct Inner { delicious: i32 }

struct Outer { om_nom_nom: Inner, thing: u32 }

fn let_members_of_members() -> i32 {
    let thing = Outer();

    let inner = thing.om_nom_nom;
    let delishus = inner.delicious;

    if (thing.thing != u32(delishus)) {
        // LOL
    }

    return thing.om_nom_nom.delicious;
}

fn var_members_of_members() -> i32 {
    var thing = Outer();

    var inner = thing.om_nom_nom;
    var delishus = inner.delicious;

    if (thing.thing != u32(delishus)) {
        // LOL
    }

    return thing.om_nom_nom.delicious;
}

@compute @workgroup_size(1)
fn foo_compute() {
    assign_through_ptr();
    assign_to_ptr_components();
    index_ptr(true);
    member_ptr();
    let_members_of_members();
    var_members_of_members();
}
