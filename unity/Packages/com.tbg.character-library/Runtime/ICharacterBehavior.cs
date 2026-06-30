namespace TBG.CharacterLibrary
{
    public interface ICharacterBehavior
    {
        void EnterIdle();

        void PlaySpawn();

        void PlayAttack();

        void PlayDeath();
    }
}
