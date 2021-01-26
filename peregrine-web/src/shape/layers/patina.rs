use super::super::core::directcolourdraw::DirectColourDraw;
use super::super::core::spotcolourdraw::SpotColourDraw;
use crate::webgl::{ ProcessBuilder, SourceInstrs, Uniform, Attribute, GLArity, Varying, Statement };
use peregrine_core::{ DirectColour };
use super::consts::{ PR_LOW, PR_DEF };

pub(super) enum PatinaAccessorVariety { Direct, Spot }

impl PatinaAccessorVariety {
    pub const COUNT : usize = 2;

    pub fn get_index(&self) -> usize {
        match self {
            PatinaAccessorVariety::Direct => 0,
            PatinaAccessorVariety::Spot => 1
        }
    }

    pub fn get_source(&self) -> SourceInstrs {
        SourceInstrs::new(
            match self {
                PatinaAccessorVariety::Direct => vec![
                    Uniform::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity"),
                    Attribute::new(PR_LOW,GLArity::Vec3,"aVertexColour"),
                    Varying::new(PR_LOW,GLArity::Vec3,"vColour"),
                    Statement::new_vertex("vColour = vec3(aVertexColour)"),
                    Statement::new_fragment("gl_FragColor = vec4(vColour,uOpacity)")
                ],
                PatinaAccessorVariety::Spot => vec![
                    Uniform::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity"),
                    Uniform::new_fragment(PR_LOW,GLArity::Vec3,"uColour"),
                    Statement::new_fragment("gl_FragColor = vec4(uColour,uOpacity)")
                ],
                /*
                PaintSkin::Texture => vec![
                    Uniform::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity"),
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
                */
            }
        )
    }
}

pub(super) enum PatinaAccessor {
    Direct(DirectColourDraw),
    Spot(SpotColourDraw)
}

#[derive(Clone)]
pub enum PatinaAccessorName { Direct, Spot(DirectColour) }

impl PatinaAccessorName {
    pub(super) fn make_accessor(&self, process: &ProcessBuilder) -> anyhow::Result<PatinaAccessor> {
        Ok(match self {
            PatinaAccessorName::Direct => PatinaAccessor::Direct(DirectColourDraw::new(process)?),
            PatinaAccessorName::Spot(colour) => PatinaAccessor::Spot(SpotColourDraw::new(process,colour)?)
        })
    }

    pub(super) fn get_variety(&self) -> PatinaAccessorVariety {
        match self {
            PatinaAccessorName::Direct => PatinaAccessorVariety::Direct,
            PatinaAccessorName::Spot(_) => PatinaAccessorVariety::Spot
        }
    }
}
