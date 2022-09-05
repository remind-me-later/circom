use super::{Tree, DAG};
use circom_algebra::algebra::Constraint;
use circom_algebra::num_bigint::BigInt;
use circom_constraint_writers::debug_writer::DebugWriter;
use circom_constraint_writers::json_writer::ConstraintJSON;
use serde_json::Value;
use std::collections::HashMap;
use std::io;

type C = Constraint<usize>;

fn transform_constraint_to_json(constraint: &C) -> Value {
    Value::Array(vec![
        hashmap_as_json(constraint.a()),
        hashmap_as_json(constraint.b()),
        hashmap_as_json(constraint.c()),
    ])
}
fn hashmap_as_json(values: &HashMap<usize, BigInt>) -> Value {
    let mut order: Vec<&usize> = values.keys().collect();
    order.sort();
    let mut correspondence = serde_json::Value::Object(serde_json::Map::new());
    for i in order {
        let (key, value) = values.get_key_value(i).unwrap();
        let value = value.to_str_radix(10);
        correspondence[format!("{}", key)] = value.as_str().into();
    }
    correspondence
}

fn visit_tree(tree: &Tree, writer: &mut ConstraintJSON) -> io::Result<()> {
    for constraint in &tree.constraints {
        let json_value = transform_constraint_to_json(constraint);
        writer.write_constraint(&json_value.to_string())?;
    }
    for edge in Tree::get_edges(tree) {
        let subtree = Tree::go_to_subtree(tree, edge);
        visit_tree(&subtree, writer)?;
    }
    Ok(())
}

pub fn port_constraints(dag: &DAG, debug: &DebugWriter) -> io::Result<()> {
    let mut writer = debug.build_constraints_file()?;
    visit_tree(&Tree::new(dag), &mut writer)?;
    writer.end()
}
