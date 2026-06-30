using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class CharacterInstance
    {
        public CharacterInstance(string id, GameObject root, ICharacterBehavior behavior)
        {
            Id = id;
            Root = root;
            Behavior = behavior;
        }

        public string Id { get; }

        public GameObject Root { get; }

        public ICharacterBehavior Behavior { get; }

        public void PlayAction(CharacterAction action)
        {
            switch (action)
            {
                case CharacterAction.Idle:
                    Behavior.EnterIdle();
                    break;
                case CharacterAction.Spawn:
                    Behavior.PlaySpawn();
                    break;
                case CharacterAction.Attack:
                    Behavior.PlayAttack();
                    break;
                case CharacterAction.Death:
                    Behavior.PlayDeath();
                    break;
                case CharacterAction.Walk:
                    Behavior.PlayWalk();
                    break;
                case CharacterAction.Ultimate:
                    Behavior.PlayUltimate();
                    break;
            }
        }
    }
}
