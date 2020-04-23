const BAR_LEN: usize = 70;

fn time_nanos(t: std::time::SystemTime) -> u128 {
    match t.duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_nanos(),
        Err(_) => panic!(),
    }
}

pub fn draw_stdout(spans: Vec<crate::Span>) {
    let mut children = std::collections::HashMap::new();
    let mut spans_map = std::collections::HashMap::new();
    let mut root = None;
    for span in spans {
        let start = time_nanos(span.start_time);
        let end = time_nanos(span.end_time);
        assert_eq!(
            spans_map.insert(span.id, (start, end - start)),
            None,
            "duplicated id {}",
            span.id
        );

        if let Some(parent) = span.parent {
            children.entry(parent).or_insert(vec![]).push(span.id);
        } else {
            root = Some(span.id);
        }
    }

    let root = root.expect("can not find root");
    let pivot = spans_map.get(&root).unwrap().0;
    let factor = BAR_LEN as f64 / spans_map.get(&root).unwrap().1 as f64;

    draw_rec(root, pivot, factor, &children, &spans_map);
}

fn draw_rec(
    cur_id: usize,
    pivot: u128,
    factor: f64,
    children_map: &std::collections::HashMap<usize, Vec<usize>>,
    spans_map: &std::collections::HashMap<usize, (u128, u128)>,
) {
    let (start, duration) = *spans_map.get(&cur_id).expect("can not get span");

    // draw leading space
    let leading_space_len = ((start - pivot) as f64 * factor) as usize;
    print!("{: <1$}", "", leading_space_len);

    // draw bar
    let bar_len = (duration as f64 * factor) as usize;
    print!("{:=<1$}", "", bar_len);

    // draw tailing space
    let tailing_space_len = BAR_LEN - bar_len - leading_space_len + 1;
    print!("{: <1$}", "", tailing_space_len);

    println!("{:6.2} ms", duration as f64 / 1000000f64);

    if let Some(children) = children_map.get(&cur_id) {
        for child in children {
            draw_rec(*child, pivot, factor, &children_map, &spans_map);
        }
    }
}
