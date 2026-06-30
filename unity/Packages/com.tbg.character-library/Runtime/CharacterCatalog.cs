using TBG.CharacterSystem.Domain;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class CharacterCatalog
    {
        private readonly CharacterDefinitionLoader _loader;
        private readonly ProceduralCharacterBuilder _proceduralBuilder;
        private readonly Mesh3DCharacterFactory _mesh3DFactory;

        public CharacterCatalog()
            : this(new CharacterDefinitionLoader(), new ProceduralCharacterBuilder(), new Mesh3DCharacterFactory())
        {
        }

        public CharacterCatalog(
            CharacterDefinitionLoader loader,
            ProceduralCharacterBuilder proceduralBuilder,
            Mesh3DCharacterFactory mesh3DFactory)
        {
            _loader = loader;
            _proceduralBuilder = proceduralBuilder;
            _mesh3DFactory = mesh3DFactory;
        }

        public CharacterInstance Spawn(string id, Transform parent)
        {
            var definition = _loader.Load(id);
            var root = definition.AssetMode == CharacterAssetMode.Mesh3D
                ? _mesh3DFactory.Create(definition, parent)
                : SpawnProcedural(definition, parent);
            var behavior = new BehaviorPlayer(root);
            return new CharacterInstance(id, root, behavior);
        }

        public void Release(CharacterInstance instance)
        {
            if (instance?.Root == null)
            {
                return;
            }

            Object.Destroy(instance.Root);
        }

        private GameObject SpawnProcedural(LoadedCharacterDefinition definition, Transform parent)
        {
            var scriptable = ScriptableObject.CreateInstance<CharacterDefinition>();
            scriptable.ApplyData(definition.ProceduralData);
            var root = _proceduralBuilder.Build(scriptable);
            root.transform.SetParent(parent, false);
            Object.Destroy(scriptable);
            return root;
        }
    }
}
