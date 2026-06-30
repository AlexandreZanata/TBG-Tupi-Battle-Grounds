using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class BehaviorPlayer : ICharacterBehavior
    {
        private readonly Animator _animator;

        public BehaviorPlayer(GameObject root)
        {
            _animator = root.GetComponentInChildren<Animator>();
        }

        public void EnterIdle() => SetTrigger("Idle");

        public void PlaySpawn() => SetTrigger("Spawn");

        public void PlayAttack() => SetTrigger("Attack");

        public void PlayDeath() => SetTrigger("Death");

        private void SetTrigger(string triggerName)
        {
            if (_animator == null)
            {
                return;
            }

            _animator.SetTrigger(triggerName);
        }
    }
}
