use bevy::log::info;

use super::*;

pub struct Commands;

impl HostCommands for WasmHost {
    fn spawn(&mut self, _self: Resource<Commands>, components: Vec<String>) -> Result<()> {
        let State::RunSystem { commands } = self.access() else {
            bail!("commands resource is only accessible when running systems")
        };

        let entity_commands = commands.spawn_empty();
        for _componet in components {
            bail!("Unimplemented");
        }

        info!("Spawning! {}", entity_commands.id());

        Ok(())
    }

    fn drop(&mut self, _rep: Resource<Commands>) -> Result<()> {
        Ok(())
    }
}
