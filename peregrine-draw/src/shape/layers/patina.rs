use enum_iterator::Sequence;
use peregrine_data::DirectColour;

use super::super::core::directcolourdraw::{ DirectColourDraw, DirectProgram };
use super::super::core::texture::{ TextureDraw, TextureProgram };
use crate::shape::core::spotcolourdraw::{SpotColourDraw, SpotProgram};
use crate::webgl::{FlatId, SetFlag};
use crate::webgl::{ SourceInstrs, UniformProto, AttributeProto, GLArity, Varying, Statement, ProgramBuilder, TextureProto };
use super::consts::{ PR_LOW, PR_DEF };
use crate::util::message::Message;

pub(crate) enum PatinaAdder {
    Direct(DirectProgram),
    Spot(SpotProgram),
    Texture(TextureProgram),
    FreeTexture(TextureProgram)
}

impl PatinaAdder {
    pub(super) fn make_patina_process(&self) -> Result<PatinaProcess,Message> {
        Ok(match self {
            PatinaAdder::Direct(v) => PatinaProcess::Direct(DirectColourDraw::new(v)?),
            PatinaAdder::Texture(v) => PatinaProcess::Texture(TextureDraw::new(v,false)?),
            PatinaAdder::FreeTexture(v) => PatinaProcess::FreeTexture(TextureDraw::new(v,true)?),
            PatinaAdder::Spot(v) => PatinaProcess::Spot(SpotColourDraw::new(v)?)
        })
    }
}

#[derive(Clone,Debug,Hash,PartialEq,Eq,Sequence)]
pub(crate) enum PatinaProgramName { Direct, Spot, Texture, FreeTexture }

impl PatinaProgramName {
    pub(crate) fn key(&self) -> String {
        format!("{:?}",self)
    }
}

pub(crate) trait PatinaYielder {
    fn name(&self) -> &PatinaProcessName;
    fn make(&mut self, builder: &ProgramBuilder) -> Result<PatinaAdder,Message>;
    fn set(&mut self, program: &PatinaProcess) -> Result<(),Message>;
}

impl PatinaProgramName {
    pub(super) fn make_patina_program(&self, builder: &ProgramBuilder) -> Result<PatinaAdder,Message> {
        Ok(match self {
            PatinaProgramName::Direct => PatinaAdder::Direct(DirectProgram::new(builder)?),
            PatinaProgramName::Texture => PatinaAdder::Texture(TextureProgram::new(builder)?),
            PatinaProgramName::FreeTexture => PatinaAdder::FreeTexture(TextureProgram::new(builder)?),
            PatinaProgramName::Spot => PatinaAdder::Spot(SpotProgram::new(builder)?)
        })
    }

    pub fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(
            match self {
                PatinaProgramName::Direct => vec![
                    AttributeProto::new(PR_DEF,GLArity::Vec4,"aVertexColour"),
                    Varying::new(PR_LOW,GLArity::Vec4,"vColour"),
                    Statement::new_vertex("vColour = aVertexColour"),
                    Statement::new_fragment("gl_FragColor = vColour"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity")
                ],
                PatinaProgramName::Spot => vec![
                    UniformProto::new_fragment(PR_LOW,GLArity::Vec4,"uColour"),
                    Statement::new_fragment("gl_FragColor = uColour"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity")
                ],
                PatinaProgramName::Texture => vec![
                    TextureProto::new("uSampler","uSamplerSize","uSamplerScale"),
                    AttributeProto::new(PR_DEF,GLArity::Vec2,"aTextureCoord"),
                    AttributeProto::new(PR_DEF,GLArity::Vec2,"aMaskCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vTextureCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vMaskCoord"),
                    Statement::new_vertex("vTextureCoord = aTextureCoord"),
                    Statement::new_vertex("vMaskCoord = aMaskCoord"),
                    Statement::new_fragment("gl_FragColor = texture2D(uSampler,vTextureCoord)"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity"),
                    Statement::new_fragment("if(texture2D(uSampler,vMaskCoord).r > 0.995) discard")
                ],
                PatinaProgramName::FreeTexture => vec![
                    TextureProto::new("uSampler","uSamplerSize","uSamplerScale"),
                    AttributeProto::new(PR_DEF,GLArity::Vec2,"aTextureCoord"),
                    AttributeProto::new(PR_DEF,GLArity::Vec2,"aMaskCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vTextureCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vMaskCoord"),
                    Statement::new_vertex("vTextureCoord = aTextureCoord"),
                    Statement::new_vertex("vMaskCoord = aMaskCoord"),
                    Statement::new_fragment("gl_FragColor = texture2D(uSampler,vec2(
                            (gl_FragCoord.x/uSamplerScale.x-vOrigin.x)/uSamplerSize.x+vTextureCoord.x,
                            (vOrigin.y-gl_FragCoord.y/uSamplerScale.y)/uSamplerSize.y+vTextureCoord.y))"),
                    SetFlag::new("need-origin"),
                    Statement::new_fragment("lowp vec4 mask = texture2D(uSampler,vec2(
                        (gl_FragCoord.x/uSamplerScale.x-vOrigin.x)/uSamplerSize.x+vMaskCoord.x,
                        (vOrigin.y-gl_FragCoord.y/uSamplerScale.y)/uSamplerSize.y+vMaskCoord.y))"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity"),
                    Statement::new_fragment("if(mask.r > 0.995) discard")
                ]
            }
        )
    }
}

pub(crate) enum PatinaProcess {
    Direct(DirectColourDraw),
    Texture(TextureDraw),
    FreeTexture(TextureDraw),
    Spot(SpotColourDraw),
    None
}

// TODO texture types

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum PatinaProcessName { Direct, Spot(DirectColour), Texture(FlatId), FreeTexture(FlatId) }

impl PatinaProcessName {
    pub(super) fn get_program_name(&self) -> PatinaProgramName {
        match self {
            PatinaProcessName::Direct => PatinaProgramName::Direct,
            PatinaProcessName::Spot(_) => PatinaProgramName::Spot,
            PatinaProcessName::Texture(_) => PatinaProgramName::Texture,
            PatinaProcessName::FreeTexture(_) => PatinaProgramName::FreeTexture
        }
    }
}
