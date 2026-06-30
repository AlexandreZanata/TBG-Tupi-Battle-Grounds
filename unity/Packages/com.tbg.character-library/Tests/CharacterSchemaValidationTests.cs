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
        public void Validate_Mesh3dDemo_HasNoErrors()
        {
            var record = LoadRecord("demo_striker_01");
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Is.Empty);
            Assert.That(record.assetMode, Is.EqualTo("mesh3d"));
        }

        [Test]
        public void Validate_Mesh3dExample_HasNoErrors()
        {
            var record = LoadRecord("_schema.mesh3d.example");
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Is.Empty);
            Assert.That(record.mesh, Does.EndWith(".glb"));
        }

        [Test]
        public void Validate_Mesh3dWithoutMesh_ReturnsError()
        {
            var record = LoadRecord("demo_striker_01");
            record.mesh = null;
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Does.Contain("mesh3d assetMode requires mesh path"));
        }

        [Test]
        public void Validate_InvalidAssetMode_ReturnsError()
        {
            var record = LoadRecord("demo_striker_01");
            record.assetMode = "fbx";
            var errors = CharacterDefinitionValidator.Validate(record);
            Assert.That(errors, Does.Contain("invalid assetMode: fbx"));
        }

        [Test]
        public void Validate_TexturesObject_RequiresCoreMaps()
        {
            var record = LoadRecord("_schema.mesh3d.example");
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
        public void Load_AllDemoDefinitions_PassValidation()
        {
            var loader = new CharacterDefinitionLoader();
            foreach (var pilotId in new[] { "demo_scout_01", "demo_striker_01", "demo_guard_01" })
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
