use super::*;

pub struct System {
    pub(crate) name: String,
}

impl HostSystem for HostState {
    fn new(&mut self, name: String) -> Result<Resource<System>> {
        self.access(move |state| {
            let State::Setup { table, .. } = state else {
                bail!("Systems can only be instantiated in a setup function")
            };

            let name = name.clone();
            Ok(table.push(System { name })?)
        })
    }

    fn add_commands(&mut self, _self: Resource<System>) -> Result<()> {
        Ok(())
    }

    fn add_query(&mut self, _self: Resource<System>, _query: Vec<QueryFor>) -> Result<()> {
        Ok(())
    }

    fn before(&mut self, _self: Resource<System>, _other: Resource<System>) -> Result<()> {
        Ok(())
    }

    fn after(&mut self, _self: Resource<System>, _other: Resource<System>) -> Result<()> {
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<System>) -> Result<()> {
        Ok(())
    }
}
