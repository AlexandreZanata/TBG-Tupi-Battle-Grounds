using UnityEngine;

namespace TBG.CharacterLibrary.Locomotion
{
    /// <summary>Lightweight two-bone IK — avoids Animation Rigging package dependency.</summary>
    public static class TwoBoneIk
    {
        public static bool Solve(
            Transform upper,
            Transform lower,
            Transform end,
            Vector3 target,
            Vector3 poleHint)
        {
            if (upper == null || lower == null || end == null)
            {
                return false;
            }

            var root = upper.position;
            var mid = lower.position;
            var tip = end.position;
            var upperLen = Vector3.Distance(root, mid);
            var lowerLen = Vector3.Distance(mid, tip);
            if (upperLen < 1e-4f || lowerLen < 1e-4f)
            {
                return false;
            }

            target = ClampReach(root, target, upperLen + lowerLen - 1e-3f);
            var toTarget = target - root;
            var dist = toTarget.magnitude;
            if (dist < 1e-4f)
            {
                return false;
            }

            var cosUpper = (upperLen * upperLen + dist * dist - lowerLen * lowerLen) / (2f * upperLen * dist);
            var upperAngle = Mathf.Acos(Mathf.Clamp(cosUpper, -1f, 1f));
            var bendNormal = Vector3.Cross(toTarget, poleHint - root);
            if (bendNormal.sqrMagnitude < 1e-6f)
            {
                bendNormal = Vector3.Cross(toTarget, Vector3.up);
            }

            bendNormal.Normalize();
            var upperDir = Quaternion.AngleAxis(upperAngle * Mathf.Rad2Deg, bendNormal) * toTarget.normalized;
            var desiredMid = root + upperDir * upperLen;
            upper.rotation = Quaternion.FromToRotation(mid - root, desiredMid - root) * upper.rotation;
            lower.rotation = Quaternion.FromToRotation(tip - lower.position, target - lower.position) * lower.rotation;
            return true;
        }

        internal static Vector3 ClampReach(Vector3 root, Vector3 target, float maxReach)
        {
            var offset = target - root;
            if (offset.sqrMagnitude <= maxReach * maxReach)
            {
                return target;
            }

            return root + offset.normalized * maxReach;
        }
    }
}
