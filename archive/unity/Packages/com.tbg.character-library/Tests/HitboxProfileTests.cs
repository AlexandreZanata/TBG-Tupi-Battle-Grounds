using NUnit.Framework;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class HitboxProfileTests
    {
        [Test]
        public void DefaultBipedProfile_ExposesHeadAndTorsoColliders()
        {
            var profile = DefaultBipedHitboxProfile.Create();
            Assert.That(profile.ProfileId, Is.EqualTo(DefaultBipedHitboxProfile.ProfileIdValue));
            Assert.That(profile.Colliders, Has.Count.EqualTo(7));

            var head = profile.Colliders[0];
            Assert.That(head.BoneName, Is.EqualTo("TBG_Head"));
            Assert.That(head.Shape, Is.EqualTo(HitboxColliderShape.Sphere));

            var torso = profile.Colliders[1];
            Assert.That(torso.BoneName, Is.EqualTo("TBG_Spine1"));
            Assert.That(torso.Shape, Is.EqualTo(HitboxColliderShape.Capsule));
        }

        [Test]
        public void DefaultBipedProfile_IncludesWeaponReachOnDominantHand()
        {
            var profile = DefaultBipedHitboxProfile.Create();
            var weapon = profile.Colliders[6];
            Assert.That(weapon.BoneName, Is.EqualTo("TBG_Hand.R"));
            Assert.That(weapon.Shape, Is.EqualTo(HitboxColliderShape.Box));
        }
    }
}
