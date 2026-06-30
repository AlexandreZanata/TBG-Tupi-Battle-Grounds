using System.IO;
using NUnit.Framework;
using TBG.CharacterLibrary;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class CharacterSchemaValidationTests
    {
        [Test]
        public void Validate_Mesh3dPilot_HasNoErrors()
        {
            var record = LoadRecord("tank_human_01");
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Is.Empty);
            Assert.That(record.assetMode, Is.EqualTo("mesh3d"));
        }

        [Test]
        public void Validate_Mesh3dFixture_HasNoErrors()
        {
            var record = LoadRecord("test_mesh3d_spawn");
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Is.Empty);
            Assert.That(record.mesh, Does.EndWith(".glb"));
        }

        [Test]
        public void Validate_Mesh3dWithoutMesh_ReturnsError()
        {
            var record = LoadRecord("test_mesh3d_spawn");
            record.mesh = null;
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Does.Contain("mesh3d assetMode requires mesh path"));
        }

        [Test]
        public void Validate_InvalidAssetMode_ReturnsError()
        {
            var record = LoadRecord("tank_human_01");
            record.assetMode = "fbx";
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Does.Contain("invalid assetMode: fbx"));
        }

        [Test]
        public void Validate_TexturesObject_RequiresCoreMaps()
        {
            var record = LoadRecord("test_mesh3d_spawn");
            record.textures = new CharacterTexturesRecord
            {
                albedo = string.Empty,
                normal = "n.png",
                orm = "o.png",
            };
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Does.Contain("textures.albedo is required when textures object is present"));
        }

        [Test]
        public void Load_AllPilotDefinitions_PassValidation()
        {
            var loader = new CharacterDefinitionLoader();
            foreach (var pilotId in new[] { "tank_human_01", "dps_undead_01", "support_machine_01", "ranged_plant_01" })
            {
                var definition = loader.Load(pilotId);
                Assert.That(definition.AssetMode, Is.EqualTo(CharacterAssetMode.Mesh3D));
            }
        }

        private static CharacterDefinitionJsonRecord LoadRecord(string id)
        {
            var path = Path.Combine(CharacterPaths.DefinitionsDir, id + ".json");
            var json = File.ReadAllText(path);
            return JsonUtility.FromJson<CharacterDefinitionJsonRecord>(json);
        }
    }
}
