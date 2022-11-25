use enum_iterator::Sequence;
use peregrine_toolkit::error::Error;
use super::super::core::directcolourdraw::{ DirectColourDraw, DirectProgram };
use super::super::core::texture::{ TextureDraw, TextureProgram };
use crate::webgl::canvas::htmlcanvas::canvasinuse::CanvasInUse;
use crate::webgl::{SetFlag};
use crate::webgl::{ SourceInstrs, UniformProto, AttributeProto, GLArity, Varying, Statement, ProgramBuilder, TextureProto };
use super::consts::{ PR_LOW, PR_DEF };

pub(crate) enum PatinaAdder {
    Direct(DirectProgram),
    Texture(TextureProgram),
    FreeTexture(TextureProgram)
}

impl PatinaAdder {
    pub(super) fn make_patina_process(&self) -> Result<PatinaProcess,Error> {
        Ok(match self {
            PatinaAdder::Direct(v) => PatinaProcess::Direct(DirectColourDraw::new(v)?),
            PatinaAdder::Texture(v) => PatinaProcess::Texture(TextureDraw::new(v,false)?),
            PatinaAdder::FreeTexture(v) => PatinaProcess::FreeTexture(TextureDraw::new(v,true)?),
        })
    }
}

#[derive(Clone,Debug,Hash,PartialEq,Eq,Sequence)]
pub(crate) enum PatinaProgramName { Direct, Texture, FreeTexture }

impl PatinaProgramName {
    pub(crate) fn key(&self) -> String {
        format!("{:?}",self)
    }
}

pub(crate) trait PatinaYielder {
    fn name(&self) -> &PatinaProcessName;
    fn make(&mut self, builder: &ProgramBuilder) -> Result<PatinaAdder,Error>;
    fn set(&mut self, program: &PatinaProcess) -> Result<(),Error>;
}

impl PatinaProgramName {
    pub(super) fn make_patina_program(&self, builder: &ProgramBuilder) -> Result<PatinaAdder,Error> {
        Ok(match self {
            PatinaProgramName::Direct => PatinaAdder::Direct(DirectProgram::new(builder)?),
            PatinaProgramName::Texture => PatinaAdder::Texture(TextureProgram::new(builder)?),
            PatinaProgramName::FreeTexture => PatinaAdder::FreeTexture(TextureProgram::new(builder)?),
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
                PatinaProgramName::Texture => vec![
                    TextureProto::new("uSampler","uSamplerSize","uSamplerScale"),
                    AttributeProto::new(PR_DEF,GLArity::Vec2,"aTextureCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vTextureCoord"),
                    Statement::new_vertex("vTextureCoord = aTextureCoord"),
                    Statement::new_fragment("gl_FragColor = texture2D(uSampler,vTextureCoord)"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity"),
                ],
                PatinaProgramName::FreeTexture => vec![
                    TextureProto::new("uSampler","uSamplerSize","uSamplerScale"),
                    AttributeProto::new(PR_DEF,GLArity::Vec2,"aTextureCoord"),
                    Varying::new(PR_DEF,GLArity::Vec2,"vTextureCoord"),
                    Statement::new_vertex("vTextureCoord = aTextureCoord"),
                    UniformProto::new_fragment(PR_LOW,GLArity::Vec2,"uFreedom"),
                    SetFlag::new("need-origin"),
                    Statement::new_fragment("gl_FragColor = texture2D(uSampler,
                        vec2(
                            uFreedom.y*(gl_FragCoord.x-vOrigin.x)/uSamplerSize.x+vTextureCoord.x,
                            uFreedom.x*(gl_FragCoord.y-vOrigin.y)/uSamplerSize.y+vTextureCoord.y)
                        )"),
                    Statement::new_fragment("gl_FragColor.a = gl_FragColor.a * uOpacity"),
                ]
            }
        )
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum Freedom {
    None,
    Horizontal,
    Vertical
}

impl Freedom {
    pub(crate) fn as_gl(&self) -> (f32,f32) {
        match self {
            Freedom::None => (0.,0.),
            Freedom::Horizontal => (1.,0.),
            Freedom::Vertical => (0.,1.),
        }
    }
}

pub(crate) enum PatinaProcess {
    Direct(DirectColourDraw),
    Texture(TextureDraw),
    FreeTexture(TextureDraw)
}

// TODO texture types

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(crate) enum PatinaProcessName { Direct, Texture(CanvasInUse), FreeTexture(CanvasInUse,Freedom) }

impl PatinaProcessName {
    pub(super) fn get_program_name(&self) -> PatinaProgramName {
        match self {
            PatinaProcessName::Direct => PatinaProgramName::Direct,
            PatinaProcessName::Texture(_) => PatinaProgramName::Texture,
            PatinaProcessName::FreeTexture(_,_) => PatinaProgramName::FreeTexture
        }
    }

    pub(super) fn order(&self) -> usize {
        match self {
            PatinaProcessName::Direct => 0,
            PatinaProcessName::Texture(_) => 1,
            PatinaProcessName::FreeTexture(_,_) => 2
        }
    }
}
