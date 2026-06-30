using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class BehaviorPlayer : ICharacterBehavior
    {
        private readonly Animator _animator;
        private readonly BehaviorProfile _profile;

        public BehaviorPlayer(GameObject root, BehaviorProfile profile = null)
        {
            _animator = root.GetComponentInChildren<Animator>();
            _profile = profile ?? BehaviorProfile.CreateDefault();
        }

        public void EnterIdle() => Play(CharacterAction.Idle);

        public void PlaySpawn() => Play(CharacterAction.Spawn);

        public void PlayAttack() => Play(CharacterAction.Attack);

        public void PlayDeath() => Play(CharacterAction.Death);

        public string GetStateName(CharacterAction action) => _profile.GetStateName(action);

        public string GetTriggerName(CharacterAction action) => _profile.GetTriggerName(action);

        private void Play(CharacterAction action)
        {
            if (_animator == null)
            {
                return;
            }

            var trigger = _profile.GetTriggerName(action);
            if (!string.IsNullOrWhiteSpace(trigger) && HasParameter(trigger))
            {
                _animator.SetTrigger(trigger);
                return;
            }

            var state = _profile.GetStateName(action);
            if (!string.IsNullOrWhiteSpace(state))
            {
                _animator.CrossFade(state, 0.1f, 0);
            }
        }

        private bool HasParameter(string name)
        {
            foreach (var parameter in _animator.parameters)
            {
                if (parameter.name == name && parameter.type == AnimatorControllerParameterType.Trigger)
                {
                    return true;
                }
            }

            return false;
        }
    }
}
