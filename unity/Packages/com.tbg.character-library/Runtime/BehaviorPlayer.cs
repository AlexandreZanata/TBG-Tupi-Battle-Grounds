using System;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class BehaviorPlayer : ICharacterBehavior
    {
        private readonly Animator _animator;
        private readonly BehaviorProfile _profile;
        private int _attackIndex;

        public BehaviorPlayer(GameObject root, BehaviorProfile profile = null)
        {
            _animator = root.GetComponentInChildren<Animator>();
            _profile = profile ?? BehaviorProfile.CreateDefault();
        }

        public event Action<CharacterAction> OnClipComplete;

        public void EnterIdle() => Play(CharacterAction.Idle);

        public void PlaySpawn() => Play(CharacterAction.Spawn);

        public void PlayAttack() => PlayAttack(0);

        public void PlayAttack(int index)
        {
            _attackIndex = index;
            var action = index <= 0 ? CharacterAction.Attack : CharacterAction.Attack;
            var state = _profile.GetAttackStateName(index);
            if (!string.IsNullOrWhiteSpace(state))
            {
                CrossFadeState(state);
                return;
            }

            Play(action);
        }

        public void PlayAttackRepeat()
        {
            if (string.Equals(_profile.AttackCycle, "rotate", StringComparison.OrdinalIgnoreCase))
            {
                PlayAttack(_attackIndex);
                _attackIndex = _attackIndex == 0 ? 1 : 0;
                return;
            }

            PlayAttack(0);
        }

        public void PlayDeath() => Play(CharacterAction.Death);

        public void PlayWalk() => EnterWalk();

        public void EnterWalk()
        {
            if (TrySetBool("IsWalking", true))
            {
                return;
            }

            Play(CharacterAction.Walk);
        }

        public void ExitWalk()
        {
            if (TrySetBool("IsWalking", false))
            {
                EnterIdle();
                return;
            }

            EnterIdle();
        }

        /// <summary>
        /// ponytail: stub — Phase 04 lane controller applies WalkRoot clip root delta when enabled.
        /// </summary>
        public void SyncWalkRootMotion(Vector3 worldDelta)
        {
            if (!_profile.UseRootMotionWalk || worldDelta.sqrMagnitude <= 0f)
            {
                return;
            }

            if (TrySetBool("UseRootMotion", true))
            {
                return;
            }
        }

        public void PlayUltimate() => Play(CharacterAction.Ultimate);

        public string GetStateName(CharacterAction action) => _profile.GetStateName(action);

        public string GetTriggerName(CharacterAction action) => _profile.GetTriggerName(action);

        public void NotifyClipComplete(CharacterAction action)
        {
            OnClipComplete?.Invoke(action);
        }

        private void Play(CharacterAction action)
        {
            if (_animator == null)
            {
                return;
            }

            var trigger = _profile.GetTriggerName(action);
            if (!string.IsNullOrWhiteSpace(trigger) && HasParameter(trigger, AnimatorControllerParameterType.Trigger))
            {
                _animator.SetTrigger(trigger);
                return;
            }

            var state = _profile.GetStateName(action);
            if (!string.IsNullOrWhiteSpace(state))
            {
                CrossFadeState(state);
            }
        }

        private void CrossFadeState(string state)
        {
            _animator.CrossFade(state, 0.1f, 0);
        }

        private bool TrySetBool(string name, bool value)
        {
            if (_animator == null || !HasParameter(name, AnimatorControllerParameterType.Bool))
            {
                return false;
            }

            _animator.SetBool(name, value);
            return true;
        }

        private bool HasParameter(string name, AnimatorControllerParameterType type)
        {
            foreach (var parameter in _animator.parameters)
            {
                if (parameter.name == name && parameter.type == type)
                {
                    return true;
                }
            }

            return false;
        }
    }
}
