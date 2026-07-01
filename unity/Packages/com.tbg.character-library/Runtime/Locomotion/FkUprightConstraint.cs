using UnityEngine;

namespace TBG.CharacterLibrary.Locomotion
{
    /// <summary>World-space upright — zero torso yaw/roll after animator (mirrors web fkUprightConstraint.js).</summary>
    [DefaultExecutionOrder(100)]
    public sealed class FkUprightConstraint : MonoBehaviour
    {
        private static readonly string[] TorsoBones =
        {
            "TBG_Spine", "TBG_Spine1", "TBG_Neck", "TBG_Head",
        };

        private static readonly string[] WeaponArm =
        {
            "TBG_Shoulder.R", "TBG_UpperArm.R", "TBG_LowerArm.R", "TBG_Hand.R",
        };

        private const string StabilizeRoot = "TBG_Root";

        private Transform _root;
        private Transform[] _torso;
        private Transform[] _weaponArm;

        public static void Attach(GameObject root)
        {
            if (root == null)
            {
                return;
            }

            var constraint = root.GetComponent<FkUprightConstraint>() ?? root.AddComponent<FkUprightConstraint>();
            constraint.Bind(root.transform);
        }

        private void Bind(Transform root)
        {
            _root = FindBone(root, StabilizeRoot);
            _torso = ResolveBones(root, TorsoBones);
            _weaponArm = ResolveBones(root, WeaponArm);
        }

        private void LateUpdate()
        {
            if (_root != null)
            {
                ApplyWorldPitchOnly(_root);
            }

            foreach (var bone in _torso)
            {
                if (bone != null)
                {
                    ApplyWorldPitchOnly(bone);
                }
            }

            foreach (var bone in _weaponArm)
            {
                if (bone != null)
                {
                    ApplyWorldYawOnly(bone);
                }
            }
        }

        private static Transform[] ResolveBones(Transform root, string[] names)
        {
            var bones = new Transform[names.Length];
            for (var i = 0; i < names.Length; i++)
            {
                bones[i] = FindBone(root, names[i]);
            }

            return bones;
        }

        private static Transform FindBone(Transform root, string name)
        {
            foreach (var bone in root.GetComponentsInChildren<Transform>(true))
            {
                if (bone.name == name)
                {
                    return bone;
                }
            }

            return null;
        }

        private static void ApplyWorldPitchOnly(Transform bone)
        {
            var worldQ = bone.rotation;
            QuaternionYXZ.ToAngles(worldQ, out var x, out _, out _);
            var localQ = ParentLocalRotation(bone, QuaternionYXZ.FromAngles(x, 0f, 0f));
            bone.localRotation = localQ;
        }

        private static void ApplyWorldYawOnly(Transform bone)
        {
            var worldQ = bone.rotation;
            QuaternionYXZ.ToAngles(worldQ, out var x, out _, out var z);
            var localQ = ParentLocalRotation(bone, QuaternionYXZ.FromAngles(x, 0f, z));
            bone.localRotation = localQ;
        }

        private static Quaternion ParentLocalRotation(Transform bone, Quaternion worldQ)
        {
            if (bone.parent == null)
            {
                return worldQ;
            }

            return Quaternion.Inverse(bone.parent.rotation) * worldQ;
        }

        private static class QuaternionYXZ
        {
            public static void ToAngles(Quaternion q, out float x, out float y, out float z)
            {
                var sinY = 2f * (q.w * q.y + q.z * q.x);
                var cy = Mathf.Sqrt(Mathf.Max(1e-6f, 1f - sinY * sinY));
                x = Mathf.Atan2(2f * (q.w * q.x - q.y * q.z), cy);
                y = Mathf.Asin(Mathf.Clamp(sinY, -1f, 1f));
                z = Mathf.Atan2(2f * (q.w * q.z + q.x * q.y), cy);
            }

            public static Quaternion FromAngles(float x, float y, float z)
            {
                var cx = Mathf.Cos(x * 0.5f);
                var sx = Mathf.Sin(x * 0.5f);
                var cy = Mathf.Cos(y * 0.5f);
                var sy = Mathf.Sin(y * 0.5f);
                var cz = Mathf.Cos(z * 0.5f);
                var sz = Mathf.Sin(z * 0.5f);
                return new Quaternion(
                    sx * cy * cz + cx * sy * sz,
                    cx * sy * cz - sx * cy * sz,
                    cx * cy * sz + sx * sy * cz,
                    cx * cy * cz - sx * sy * sz);
            }
        }
    }
}
