using UnityEngine;

namespace TBG.CharacterLibrary.Locomotion
{
    /// <summary>Raycast foot plant during Walk — gated by <see cref="BehaviorProfile.UseFootIk"/>.</summary>
    public sealed class LocomotionFootIk : MonoBehaviour
    {
        private const float RayHeight = 0.45f;
        private const float RayDistance = 1.2f;

        private BehaviorProfile _profile;
        private Animator _animator;
        private LegIkChain _left;
        private LegIkChain _right;

        public static void Attach(GameObject root, BehaviorProfile profile)
        {
            if (root == null || profile == null || !profile.UseFootIk)
            {
                return;
            }

            var ik = root.GetComponent<LocomotionFootIk>() ?? root.AddComponent<LocomotionFootIk>();
            ik.Bind(profile);
        }

        private void Bind(BehaviorProfile profile)
        {
            _profile = profile;
            _animator = GetComponentInChildren<Animator>();
            _left = LegIkChain.Find(transform, "TBG_UpperLeg.L", "TBG_LowerLeg.L", "TBG_Foot.L");
            _right = LegIkChain.Find(transform, "TBG_UpperLeg.R", "TBG_LowerLeg.R", "TBG_Foot.R");
        }

        private void LateUpdate()
        {
            if (_profile == null || !_profile.UseFootIk || !IsWalking())
            {
                return;
            }

            PlantLeg(_left);
            PlantLeg(_right);
        }

        private bool IsWalking()
        {
            if (_animator == null)
            {
                return false;
            }

            foreach (var parameter in _animator.parameters)
            {
                if (parameter.name == "IsWalking" && parameter.type == AnimatorControllerParameterType.Bool)
                {
                    return _animator.GetBool("IsWalking");
                }
            }

            var state = _animator.GetCurrentAnimatorStateInfo(0);
            return state.IsName("Walk");
        }

        private void PlantLeg(LegIkChain leg)
        {
            if (!leg.IsValid)
            {
                return;
            }

            var origin = leg.Foot.position + Vector3.up * RayHeight;
            if (!Physics.Raycast(origin, Vector3.down, out var hit, RayDistance))
            {
                return;
            }

            var pole = leg.Upper.position + transform.right * (leg.IsLeft ? -0.25f : 0.25f);
            TwoBoneIk.Solve(leg.Upper, leg.Lower, leg.Foot, hit.point, pole);
        }
    }
}
