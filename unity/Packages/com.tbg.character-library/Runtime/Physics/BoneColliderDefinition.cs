using System;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    /// <summary>Bone-attached collider definition — see docs/design/physics-hitbox-spec.md.</summary>
    [Serializable]
    public sealed class BoneColliderDefinition
    {
        public string BoneName = string.Empty;
        public HitboxColliderShape Shape = HitboxColliderShape.Capsule;
        public float Radius = 0.1f;
        public float Height = 0.3f;
        public Vector3 LocalCenter = Vector3.zero;
        public Vector3 LocalSize = Vector3.one * 0.2f;

        public BoneColliderDefinition()
        {
        }

        public BoneColliderDefinition(
            string boneName,
            HitboxColliderShape shape,
            float radius = 0.1f,
            float height = 0.3f)
        {
            BoneName = boneName;
            Shape = shape;
            Radius = radius;
            Height = height;
        }
    }
}
