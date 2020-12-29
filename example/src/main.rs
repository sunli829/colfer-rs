use colfer::Message;

mod gen;

fn main() {
    let r = gen::DromedaryCase {
        pascal_case: "anc".to_string(),
    };

    let data = r.to_vec().unwrap();
    assert_eq!(gen::DromedaryCase::from_bytes(&data).unwrap(), r);
}
