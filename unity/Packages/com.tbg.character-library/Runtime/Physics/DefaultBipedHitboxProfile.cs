using System.Collections.Generic;

namespace TBG.CharacterLibrary
{
    /// <summary>Default HumanoidLight / HumanoidHeavy hitboxes — stub for Phase 04.</summary>
    public sealed class DefaultBipedHitboxProfile : IHitboxProfile
    {
        public const string ProfileIdValue = "default_biped_v1";

        private static readonly BoneColliderDefinition[] DefaultColliders =
        {
            new("TBG_Head", HitboxColliderShape.Sphere, 0.12f),
            new("TBG_Spine1", HitboxColliderShape.Capsule, 0.18f, 0.45f),
            new("TBG_UpperArm.L", HitboxColliderShape.Capsule, 0.08f, 0.28f),
            new("TBG_UpperArm.R", HitboxColliderShape.Capsule, 0.08f, 0.28f),
            new("TBG_UpperLeg.L", HitboxColliderShape.Capsule, 0.1f, 0.38f),
            new("TBG_UpperLeg.R", HitboxColliderShape.Capsule, 0.1f, 0.38f),
            new("TBG_Hand.R", HitboxColliderShape.Box, 0.05f, 0.2f)
            {
                LocalSize = new UnityEngine.Vector3(0.08f, 0.08f, 0.35f),
                LocalCenter = new UnityEngine.Vector3(0f, 0f, 0.18f),
            },
        };

        public string ProfileId => ProfileIdValue;

        public IReadOnlyList<BoneColliderDefinition> Colliders => DefaultColliders;

        public static DefaultBipedHitboxProfile Create() => new();
    }
}
