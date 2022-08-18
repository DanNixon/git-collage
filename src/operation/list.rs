use crate::config::RepositoryMapping;

pub(super) fn run(mappings: &[RepositoryMapping]) -> std::result::Result<(), usize> {
    for m in mappings {
        println!("{}", m);
    }
    Ok(())
}
