use serde_json::json;
use skylab_rust::{Project, blocks};
#[test]
fn block_membership_uses_clipped_length_and_respects_holes_and_outside_lines() {
    let p: Project = serde_json::from_value(
        json!({"version":1,"crs":{"zone":23,"south":true},"warnings":[],"polygons":[
            {"outer":[[0,0],[10,0],[10,10],[0,10],[0,0]],"holes":[[[3,3],[7,3],[7,7],[3,7],[3,3]]]},
            {"outer":[[20,0],[30,0],[30,10],[20,10],[20,0]],"holes":[]}
        ],"lines":[
            {"name":"A","points":[[1,1],[9,1]]},
            {"name":"B","points":[[21,1],[29,1]]},
            {"name":"HOLE","points":[[4,5],[6,5]]},
            {"name":"OUT","points":[[40,1],[50,1]]},
            {"name":"CROSS","points":[[8,1],[29,1]]}
        ]}),
    )
    .unwrap();
    assert_eq!(
        blocks::assign(&p),
        vec![Some(0), Some(1), None, None, Some(1)]
    );
    assert_eq!(p.lines[4].points, vec![[8., 1.], [29., 1.]]);
}
