use crate::animation::{PointDef, PointDefinition};
use crate::animation_v2::AnimationV2;
use crate::tests::DataError;

#[test]
fn test_v2_anim() -> Result<(), DataError> {
    let data = r#"{
        "_position": [
            [
                6,
                0,
                -18,
                0.125
            ],
            [
                0,
                0,
                0,
                0.375,
                "easeInOutCirc"
            ]
        ],
        "_localRotation": [
            [
                61,
                -154,
                -56,
                0.125
            ],
            [
                0,
                0,
                0,
                0.375,
                "easeInOutCirc"
            ]
        ]
    }"#;
    let anim: Result<AnimationV2, _> = serde_json::from_str(data);
    println!("animation: {anim:#?}");
    let anim = anim?;

    let round = serde_json::to_string(&anim);
    println!("animation-round: {round:#?}");
    round?;

    Ok(())
}
