using NUnit.Framework;
using TBG.CharacterLibrary;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class BodyClassClipRegistryTests
    {
        [TearDown]
        public void TearDown()
        {
            BodyClassClipRegistry.ClearForTests();
        }

        [Test]
        public void Register_SameBodyClass_StoresSingleAnimationSet()
        {
            BodyClassClipRegistry.Register("HumanoidLight", "humanoid_light_combat_v1");
            BodyClassClipRegistry.Register("HumanoidLight", "humanoid_light_combat_v1");

            Assert.That(
                BodyClassClipRegistry.TryGetRegisteredSet("HumanoidLight", out var setId),
                Is.True);
            Assert.That(setId, Is.EqualTo("humanoid_light_combat_v1"));
        }

        [Test]
        public void ResolveSharedClipDirectory_ReturnsBodyClassPath()
        {
            var path = BodyClassClipRegistry.ResolveSharedClipDirectory("HumanoidHeavy");
            Assert.That(path, Does.Contain("animations/clips/HumanoidHeavy"));
        }
    }
}
