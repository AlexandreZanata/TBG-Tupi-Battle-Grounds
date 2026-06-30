namespace TBG.CharacterLibrary
{
    public interface ICharacterBehavior
    {
        void EnterIdle();

        void PlaySpawn();

        void PlayAttack();

        void PlayAttack(int index);

        void PlayAttackRepeat();

        void PlayDeath();

        void PlayWalk();

        void EnterWalk();

        void ExitWalk();

        void PlayUltimate();

        event System.Action<CharacterAction> OnClipComplete;
    }
}
