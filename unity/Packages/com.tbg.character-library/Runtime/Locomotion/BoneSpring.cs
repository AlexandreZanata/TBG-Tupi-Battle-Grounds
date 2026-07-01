using UnityEngine;

namespace TBG.CharacterLibrary.Locomotion
{
    /// <summary>Subtle bone spring lag on spine/arms — ossos-inspired secondary motion spike.</summary>
    [DefaultExecutionOrder(150)]
    public sealed class BoneSpring : MonoBehaviour
    {
        private static readonly string[] SpringBones =
        {
            "TBG_Spine1", "TBG_Neck", "TBG_UpperArm.L", "TBG_UpperArm.R",
        };

        private const float Strength = 0.12f;

        private Transform[] _bones;
        private Quaternion[] _state;
        private bool _enabled;

        public static void Attach(GameObject root)
        {
            if (root == null)
            {
                return;
            }

            var spring = root.GetComponent<BoneSpring>() ?? root.AddComponent<BoneSpring>();
            spring.Bind(root.transform);
        }

        public void SetEnabled(bool value)
        {
            _enabled = value;
            if (_enabled && _bones != null && _state != null)
            {
                for (var i = 0; i < _bones.Length; i++)
                {
                    if (_bones[i] != null)
                    {
                        _state[i] = _bones[i].localRotation;
                    }
                }
            }
        }

        private void Bind(Transform root)
        {
            _bones = new Transform[SpringBones.Length];
            _state = new Quaternion[SpringBones.Length];
            for (var i = 0; i < SpringBones.Length; i++)
            {
                _bones[i] = FindBone(root, SpringBones[i]);
                _state[i] = _bones[i] != null ? _bones[i].localRotation : Quaternion.identity;
            }
        }

        private void LateUpdate()
        {
            if (!_enabled || _bones == null)
            {
                return;
            }

            var t = 1f - Mathf.Exp(-10f * Strength * Time.deltaTime);
            for (var i = 0; i < _bones.Length; i++)
            {
                var bone = _bones[i];
                if (bone == null)
                {
                    continue;
                }

                _state[i] = Quaternion.Slerp(_state[i], bone.localRotation, t);
                bone.localRotation = _state[i];
            }
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
    }
}
