use super::*;

pub struct Query;

impl HostQuery for HostState {
    fn iter(&mut self, __self: Resource<Query>) -> Result<Option<Vec<Resource<Component>>>> {
        bail!("Unimplemented")
    }

    fn drop(&mut self, _rep: Resource<Query>) -> Result<()> {
        Ok(())
    }
}
