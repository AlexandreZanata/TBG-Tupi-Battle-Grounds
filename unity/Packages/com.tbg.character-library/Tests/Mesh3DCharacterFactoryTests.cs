using System.IO;
using NUnit.Framework;
using TBG.CharacterLibrary;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class Mesh3DCharacterFactoryTests
    {
        private Transform _parent;

        [SetUp]
        public void SetUp()
        {
            _parent = new GameObject("Mesh3DFactoryParent").transform;
        }

        [TearDown]
        public void TearDown()
        {
            if (_parent != null)
            {
                Object.DestroyImmediate(_parent.gameObject);
            }
        }

        [Test]
        public void Load_TestFixture_InstantiatesMeshChild()
        {
            var loader = new CharacterDefinitionLoader();
            var definition = loader.Load("test_mesh3d_spawn");
            var meshPath = Path.Combine(CharacterPaths.GetRepoRoot(), definition.MeshPath);
            var factory = new Mesh3DCharacterFactory();

            var root = factory.Load(meshPath, definition, _parent);

            Assert.That(root.transform.parent, Is.EqualTo(_parent));
            Assert.That(root.GetComponentsInChildren<MeshFilter>().Length, Is.GreaterThan(0));
            Assert.That(root.transform.localScale.y, Is.EqualTo(definition.ProceduralData.heightScale).Within(0.001f));

            Object.DestroyImmediate(root);
        }

        [Test]
        public void Create_AttachesAnimatorComponent()
        {
            var loader = new CharacterDefinitionLoader();
            var definition = loader.Load("test_mesh3d_spawn");
            var factory = new Mesh3DCharacterFactory();
            var root = factory.Create(definition, _parent);

            Assert.That(root.GetComponentInChildren<Animator>(), Is.Not.Null);

            Object.DestroyImmediate(root);
        }

        [Test]
        public void Load_MissingGlb_ThrowsFileNotFound()
        {
            var loader = new CharacterDefinitionLoader();
            var definition = loader.Load("test_mesh3d_missing_glb");
            var factory = new Mesh3DCharacterFactory();
            var missingPath = Path.Combine(CharacterPaths.GetRepoRoot(), definition.MeshPath);

            Assert.Throws<FileNotFoundException>(() => factory.Load(missingPath, definition, _parent));
        }
    }
}
