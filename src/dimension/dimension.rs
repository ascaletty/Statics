use crate::structs::Member;
use crate::structs::Truss;
pub fn update_truss(mut truss: Truss, m0: Member) {
    let member = truss.edges.iter().find(|x| x.id == m0).unwrap();
    let adj_members = truss
        .edges
        .iter()
        .filter(|x| x.start.id == member.start.id || x.end.id == member.end.id);
    for member in adj_members {}
}
