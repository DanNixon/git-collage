use crate::config::RepositoryMapping;

pub(crate) fn run(mappings: &[RepositoryMapping]) -> std::result::Result<(), usize> {
    for m in mappings {
        println!("{}", m);
    }
    Ok(())
}
