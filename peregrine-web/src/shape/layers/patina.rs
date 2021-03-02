use anyhow::bail;
use super::super::core::directcolourdraw::{ DirectColourDraw, DirectProgram };
use super::super::core::spotcolourdraw::{ SpotColourDraw, SpotProgram };
use super::super::core::texture::{ TextureDraw, TextureProgram };
use crate::webgl::FlatId;
use crate::webgl::{ ProtoProcess, SourceInstrs, Uniform, Attribute, GLArity, Varying, Statement, Program };
use peregrine_core::{ DirectColour };
use super::consts::{ PR_LOW, PR_DEF };

pub(crate) enum PatinaProgram {
    Direct(DirectProgram),
    Spot(SpotProgram),
    Texture(TextureProgram)
}

impl PatinaProgram {
    pub(super) fn make_patina_process(&self, process: &ProtoProcess, skin: &PatinaProcessName) -> anyhow::Result<PatinaProcess> {
        Ok(match self {
            PatinaProgram::Direct(v) => PatinaProcess::Direct(DirectColourDraw::new(process,v)?),
            PatinaProgram::Texture(v) => {
                match skin {
                    PatinaProcessName::Texture(_) => PatinaProcess::Texture(TextureDraw::new(process,v)?),
                    _ => bail!("unexpected type mismatch")
                }
            },
            PatinaProgram::Spot(v) => {
                match skin {
                    PatinaProcessName::Spot(colour) => PatinaProcess::Spot(SpotColourDraw::new(process,colour,v)?),
                    _ => bail!("unexpected type mismatch")
                }
            }
        })
    }
}

pub(crate) enum PatinaProgramName { Direct, Spot, Texture }

impl PatinaProgramName {
    pub const COUNT : usize = 3;

    pub fn get_index(&self) -> usize {
        match self {
            PatinaProgramName::Direct => 0,
            PatinaProgramName::Spot => 1,
            PatinaProgramName::Texture => 2
        }
    }

    pub(super) fn make_patina_program(&self, program: &Program) -> anyhow::Result<PatinaProgram> {
        Ok(match self {
            PatinaProgramName::Direct => PatinaProgram::Direct(DirectProgram::new(program)?),
            PatinaProgramName::Spot => PatinaProgram::Spot(SpotProgram::new(program)?),
            PatinaProgramName::Texture => PatinaProgram::Texture(TextureProgram::new(program)?),
        })
    }

    pub fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(
            match self {
                PatinaProgramName::Direct => vec![
                    Attribute::new(PR_LOW,GLArity::Vec3,"aVertexColour"),
                    Varying::new(PR_LOW,GLArity::Vec3,"vColour"),
                    Statement::new_vertex("vColour = vec3(aVertexColour)"),
                    Statement::new_fragment("gl_FragColor = vec4(vColour,uOpacity)")
                ],
                PatinaProgramName::Spot => vec![
                    Uniform::new_fragment(PR_LOW,GLArity::Vec3,"uColour"),
                    Statement::new_fragment("gl_FragColor = vec4(uColour,uOpacity)")
                ],
                PatinaProgramName::Texture => vec![
                    Uniform::new_fragment(PR_DEF,GLArity::Sampler2D,"uSampler"),
                    Attribute::new(PR_LOW,GLArity::Vec2,"aTextureCoord"),
                    Attribute::new(PR_LOW,GLArity::Vec2,"aMaskCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vTextureCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vMaskCoord"),
                    Statement::new_vertex("vTextureCoord = aTextureCoord"),
                    Statement::new_vertex("vMaskCoord = aMaskCoord"),
                    Statement::new_fragment("gl_FragColor = texture2d(uSampler,vTextureCoord)"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity"),
                    Statement::new_fragment("if(texture2D(uSampler,vMaskCoord).r > 0.95) discard")
                ]
            }
        )
    }
}

pub(super) enum PatinaProcess {
    Direct(DirectColourDraw),
    Spot(SpotColourDraw),
    Texture(TextureDraw)
}

// TODO texture types

#[derive(Clone)]
pub enum PatinaProcessName { Direct, Spot(DirectColour), Texture(FlatId) }

impl PatinaProcessName {
    pub(super) fn get_program_name(&self) -> PatinaProgramName {
        match self {
            PatinaProcessName::Direct => PatinaProgramName::Direct,
            PatinaProcessName::Spot(_) => PatinaProgramName::Spot,
            PatinaProcessName::Texture(_) => PatinaProgramName::Texture
        }
    }
}
