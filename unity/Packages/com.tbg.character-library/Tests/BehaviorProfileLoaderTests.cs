using System.IO;
using NUnit.Framework;
using TBG.CharacterLibrary;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class BehaviorProfileLoaderTests
    {
        [TestCase("tank_melee_v1", "Tank", "HumanoidHeavy", "AttackHeavy")]
        [TestCase("dps_melee_v1", "DPS", "Organic", "AttackFast")]
        [TestCase("support_cast_v1", "Support", "Mechanical", "AttackCast")]
        [TestCase("ranged_shot_v1", "Ranged", "HumanoidLight", "AttackRanged")]
        [TestCase("control_cast_v1", "Control", "HumanoidLight", "AttackRanged")]
        public void Load_ArchetypeProfile_MapsActions(
            string profileId,
            string archetype,
            string bodyClass,
            string attackState)
        {
            var loader = new BehaviorProfileLoader();
            var profile = loader.Load(profileId);

            Assert.That(profile.Id, Is.EqualTo(profileId));
            Assert.That(profile.Archetype, Is.EqualTo(archetype));
            Assert.That(profile.BodyClass, Is.EqualTo(bodyClass));
            Assert.That(profile.GetStateName(CharacterAction.Attack), Is.EqualTo(attackState));
            Assert.That(profile.GetTriggerName(CharacterAction.Attack), Is.EqualTo("Attack"));
            Assert.That(profile.AttackHitMs, Is.GreaterThan(profile.AttackWindupMs));
        }

        [Test]
        public void Load_TankProfile_MapsWalkAndUltimate()
        {
            var profile = new BehaviorProfileLoader().Load("tank_melee_v1");

            Assert.That(profile.GetStateName(CharacterAction.Walk), Is.EqualTo("Walk"));
            Assert.That(profile.GetStateName(CharacterAction.Ultimate), Is.EqualTo("Ultimate"));
            Assert.That(profile.UltimateDurationMs, Is.GreaterThan(profile.UltimateWindupMs));
        }

        [Test]
        public void Load_MissingProfile_ThrowsFileNotFound()
        {
            var loader = new BehaviorProfileLoader();
            Assert.Throws<FileNotFoundException>(() => loader.Load("missing_behavior_v1"));
        }

        [Test]
        public void Validate_ProfileWithoutAttackAction_ReturnsError()
        {
            var record = LoadRecord("tank_melee_v1");
            record.actions.attack = string.Empty;
            var errors = BehaviorProfileLoader.Validate(record);
            Assert.That(errors, Does.Contain("actions.attack is required"));
        }

        private static BehaviorProfileJsonRecord LoadRecord(string profileId)
        {
            var path = Path.Combine(CharacterPaths.BehaviorsDir, profileId + ".json");
            var json = File.ReadAllText(path);
            return JsonUtility.FromJson<BehaviorProfileJsonRecord>(json);
        }
    }

    public sealed class BehaviorPlayerTests
    {
        [Test]
        public void PlayAttack_WithProfile_DoesNotThrow()
        {
            var root = new GameObject("BehaviorTest");
            root.AddComponent<Animator>();
            var profile = new BehaviorProfileLoader().Load("tank_melee_v1");
            var player = new BehaviorPlayer(root, profile);

            Assert.That(player.GetStateName(CharacterAction.Attack), Is.EqualTo("AttackHeavy"));
            Assert.DoesNotThrow(() => player.PlayAttack());

            Object.DestroyImmediate(root);
        }

        [Test]
        public void PlayWalkAndUltimate_WithProfile_DoesNotThrow()
        {
            var root = new GameObject("BehaviorWalkTest");
            root.AddComponent<Animator>();
            var profile = new BehaviorProfileLoader().Load("tank_melee_v1");
            var player = new BehaviorPlayer(root, profile);

            Assert.That(player.GetStateName(CharacterAction.Walk), Is.EqualTo("Walk"));
            Assert.That(player.GetStateName(CharacterAction.Ultimate), Is.EqualTo("Ultimate"));
            Assert.DoesNotThrow(() => player.PlayWalk());
            Assert.DoesNotThrow(() => player.PlayUltimate());

            Object.DestroyImmediate(root);
        }

        [Test]
        public void SyncWalkRootMotion_WhenDisabled_IsNoOp()
        {
            var root = new GameObject("BehaviorRootMotionTest");
            root.AddComponent<Animator>();
            var profile = new BehaviorProfileLoader().Load("tank_melee_v1");
            var player = new BehaviorPlayer(root, profile);

            Assert.That(profile.UseRootMotionWalk, Is.False);
            Assert.DoesNotThrow(() => player.SyncWalkRootMotion(new Vector3(0f, 0f, 1f)));

            Object.DestroyImmediate(root);
        }

        [Test]
        public void Load_DefaultProfiles_DisableFootIk()
        {
            foreach (var profileId in new[] { "tank_melee_v1", "ranged_shot_v1" })
            {
                var profile = new BehaviorProfileLoader().Load(profileId);
                Assert.That(profile.UseFootIk, Is.False, profileId);
            }
        }
    }
}
