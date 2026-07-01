using UnityEngine;

namespace TBG.CharacterLibrary.Locomotion
{
    internal readonly struct LegIkChain
    {
        public readonly Transform Upper;
        public readonly Transform Lower;
        public readonly Transform Foot;
        public readonly bool IsLeft;

        public bool IsValid => Upper != null && Lower != null && Foot != null;

        public static LegIkChain Find(Transform root, string upper, string lower, string foot)
        {
            return new LegIkChain(
                FindBone(root, upper),
                FindBone(root, lower),
                FindBone(root, foot),
                foot.Contains(".L"));
        }

        private LegIkChain(Transform upper, Transform lower, Transform foot, bool isLeft)
        {
            Upper = upper;
            Lower = lower;
            Foot = foot;
            IsLeft = isLeft;
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
