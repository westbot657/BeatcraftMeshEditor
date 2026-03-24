use crate::data::{NormalId, UvId, VertexId};
use crate::editor::DataSwap;
use crate::light_mesh::LightMesh;
use anyhow::Result;

impl LightMesh {

    pub fn rename_vertex(&mut self, part: &str, swap: &DataSwap<VertexId>) -> Result<()> {
        if let Some(part) = self.parts.get(part) {

            


        }
        Ok(())
    }

    pub fn rename_uv(&mut self, part: &str, swap: &DataSwap<UvId>) -> Result<()> {

        Ok(())
    }

    pub fn rename_normal(&mut self, part: &str, swap: &DataSwap<NormalId>) -> Result<()> {

        Ok(())
    }

}





