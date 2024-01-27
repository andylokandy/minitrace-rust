// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

use minitrace::collector::Config;
use minitrace::collector::ConsoleReporter;
use minitrace::collector::TestReporter;
use minitrace::prelude::*;

fn main() {
    let (reporter, spans) = TestReporter::new();
    minitrace::set_reporter(reporter, Config::default());

    {
        let parent = SpanContext::random();
        let root = Span::root("root", parent).with_property(|| ("k1", "v1"));
        let _g = root.set_local_parent();
        {
            let _g = LocalSpan::enter_with_local_parent("child");
            func1(1);
            func1(1);
        }
        func2(2);
    }

    minitrace::flush();

    let spans = spans.lock().clone();
    dbg!(&spans);
    let mut tree = build_tree(spans).unwrap();
    dbg!(&tree);
    let has_event = remove_no_event(&mut tree);
    dbg!(&tree);
    sort_tree(&mut tree);
    dbg!(&tree);
    print_tree(&tree, "".to_string(), true, true);
}

#[trace(short_name = true, properties = { "i": "{i}"})]
fn func1(i: u64) {
    std::thread::sleep(Duration::from_millis(i));
    Event::add_to_local_parent("event1", || [("k1".into(), "v1".into())]);
    func2(i+1);
}

#[trace(short_name = true, properties = { "i": "{i}"})]
fn func2(i: u64) {
    std::thread::sleep(Duration::from_millis(i));
}

#[derive(Debug, Clone)]
struct TreeNode {
    record: SpanRecord,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn new(record: SpanRecord) -> Self {
        TreeNode {
            record,
            children: Vec::new(),
        }
    }
}

fn build_tree(spans: Vec<SpanRecord>) -> Option<TreeNode> {
    let mut raw = HashMap::new();
    for span in spans {
        raw.entry(span.parent_id)
            .or_insert_with(Vec::new)
            .push(span);
    }
    build_sub_tree(SpanId::default(), &raw).pop()
}

fn build_sub_tree(parent_id: SpanId, raw: &HashMap<SpanId, Vec<SpanRecord>>) -> Vec<TreeNode> {
    let mut trees = Vec::new();
    if let Some(records) = raw.get(&parent_id) {
        for record in records {
            let mut tree = TreeNode::new(record.clone());
            tree.children = build_sub_tree(record.span_id, raw);
            trees.push(tree);
        }
    }
    trees
}

fn remove_no_event(node: &mut TreeNode) -> bool {
    if !node.record.events.is_empty() {
        return true;
    }
    node.children.retain_mut(|child| remove_no_event(child));
    !node.children.is_empty()
}

fn sort_tree(node: &mut TreeNode) {
    node.record
        .events
        .sort_by_key(|event| event.timestamp_unix_ns);
    node.children
        .sort_by_key(|child| child.record.begin_time_unix_ns);
    for child in &mut node.children {
        sort_tree(child);
    }
}

fn print_tree(node: & TreeNode, prefix: String, is_last: bool, is_root: bool) {
    if is_root {
        println!(
            "{}{}",
            node.record.name,
            format_properties(&node.record.properties)
        );
    } else {
        let connector = if is_last { "╰─ " } else { "├─ " };
        println!(
            "{}{}{}{}",
            prefix,
            connector,
            node.record.name,
            format_properties(&node.record.properties)
        );
    }

    let new_prefix = if is_last { "   " } else { "│  " };
    let prefix = prefix + new_prefix;

    let count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        print_tree(child, prefix.clone(), i + 1 == count, false);
    }
}

fn format_properties(properties: &[(Cow<'static, str>, Cow<'static, str>)]) -> String {
    if properties.is_empty() {
        return "".to_string();
    }
    let props: Vec<String> = properties
        .iter()
        .map(|(k, v)| format!("\"{}\": \"{}\"", k, v))
        .collect();
    format!(" {{ {} }}", props.join(", "))
}
