using NUnit.Framework;
using TBG.CharacterLibrary;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class AnimationSetLoaderTests
    {
        [Test]
        public void Load_HumanoidHeavySet_ResolvesWalkClip()
        {
            var loader = new AnimationSetLoader();
            var manifest = loader.Load("humanoid_heavy_combat_v1");

            Assert.That(manifest.Id, Is.EqualTo("humanoid_heavy_combat_v1"));
            Assert.That(manifest.BodyClass, Is.EqualTo("HumanoidHeavy"));
            Assert.That(manifest.ResolveClipName("walk"), Is.EqualTo("Walk"));
            Assert.That(manifest.ResolveClipName("attack2"), Is.EqualTo("Attack2"));
        }

        [Test]
        public void Load_HumanoidLightSet_ResolvesAttackClip()
        {
            var manifest = new AnimationSetLoader().Load("humanoid_light_combat_v1");
            Assert.That(manifest.BodyClass, Is.EqualTo("HumanoidLight"));
            Assert.That(manifest.ResolveClipName("attack"), Is.EqualTo("Attack"));
        }
    }
}
