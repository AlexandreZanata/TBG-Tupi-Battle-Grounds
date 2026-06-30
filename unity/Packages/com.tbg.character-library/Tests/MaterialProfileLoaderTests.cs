using System.IO;
using NUnit.Framework;
using TBG.CharacterLibrary;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class MaterialProfileLoaderTests
    {
        [TestCase("humans_pbr_v1", "Humans")]
        [TestCase("undead_pbr_v1", "Undead")]
        [TestCase("machines_pbr_v1", "Machines")]
        [TestCase("plants_pbr_v1", "Plants")]
        public void Load_FactionProfile_ReturnsExpectedShader(string profileId, string faction)
        {
            var loader = new MaterialProfileLoader();
            var profile = loader.Load(profileId);

            Assert.That(profile.Id, Is.EqualTo(profileId));
            Assert.That(profile.Faction, Is.EqualTo(faction));
            Assert.That(profile.ShaderName, Is.EqualTo("TBG/CharacterPBR"));
            Assert.That(profile.TextureSlots, Does.Contain("albedo"));
            Assert.That(profile.TextureSlots, Does.Contain("normal"));
            Assert.That(profile.TextureSlots, Does.Contain("orm"));
        }

        [Test]
        public void Load_MissingProfile_ThrowsFileNotFound()
        {
            var loader = new MaterialProfileLoader();
            Assert.Throws<FileNotFoundException>(() => loader.Load("missing_profile_v1"));
        }

        [Test]
        public void CreateMaterial_UsesCharacterPbrShader()
        {
            var loader = new MaterialProfileLoader();
            var profile = loader.Load("humans_pbr_v1");
            var material = loader.CreateMaterial(profile);

            Assert.That(material.shader.name, Is.EqualTo("TBG/CharacterPBR"));
            Assert.That(material.GetFloat("_Metallic"), Is.EqualTo(profile.Metallic).Within(0.001f));
            Assert.That(material.GetFloat("_Smoothness"), Is.EqualTo(profile.Smoothness).Within(0.001f));

            Object.DestroyImmediate(material);
        }

        [Test]
        public void Validate_ProfileWithoutOrmSlot_ReturnsError()
        {
            var record = LoadRecord("humans_pbr_v1");
            record.textureSlots = new[] { "albedo", "normal" };
            var errors = MaterialProfileLoader.Validate(record);
            Assert.That(errors, Does.Contain("textureSlots must include orm"));
        }

        private static MaterialProfileJsonRecord LoadRecord(string profileId)
        {
            var path = Path.Combine(CharacterPaths.MaterialsDir, profileId + ".json");
            var json = File.ReadAllText(path);
            return JsonUtility.FromJson<MaterialProfileJsonRecord>(json);
        }
    }
}
