using TBG.CharacterSystem.Domain;
using UnityEngine;

namespace TBG.CharacterSystem.Runtime
{
    public class ProceduralCharacterBuilder
    {
        private readonly AccessoryAttacher _accessoryAttacher = new AccessoryAttacher();

        public GameObject Build(CharacterDefinition definition)
        {
            var scale = EvolutionVisualModifiers.GetScaleMultiplier(definition.evolutionStage);
            var root = new GameObject(definition.characterName);
            var bodyWidth = 0.6f * definition.bodyWidthScale * scale;

            var head = CreatePart(PrimitiveType.Cube, "Head", root.transform);
            head.localScale = Vector3.one * (0.35f * definition.headScale * scale);
            head.localPosition = new Vector3(0f, 1.75f * definition.heightScale * scale, 0f);

            var torso = CreatePart(PrimitiveType.Cube, "Torso", root.transform);
            torso.localScale = new Vector3(bodyWidth, 0.9f * definition.heightScale * scale, 0.35f);
            torso.localPosition = new Vector3(0f, 1.0f * definition.heightScale * scale, 0f);

            BuildLimb(root.transform, "Arm_L", -bodyWidth * 0.55f, definition, scale);
            BuildLimb(root.transform, "Arm_R", bodyWidth * 0.55f, definition, scale);
            BuildFoot(root.transform, "Foot_L", -bodyWidth * 0.25f, bodyWidth, definition, scale);
            BuildFoot(root.transform, "Foot_R", bodyWidth * 0.25f, bodyWidth, definition, scale);

            _accessoryAttacher.Attach(root.transform, definition.accessories, bodyWidth);
            ApplyMaterial(root, definition);
            return root;
        }

        private static Transform CreatePart(PrimitiveType type, string name, Transform parent)
        {
            var part = GameObject.CreatePrimitive(type);
            part.name = name;
            part.transform.SetParent(parent, false);
            Object.Destroy(part.GetComponent<Collider>());
            return part.transform;
        }

        private static void BuildLimb(Transform parent, string name, float x, CharacterDefinition def, float scale)
        {
            var limb = CreatePart(PrimitiveType.Capsule, name, parent);
            limb.localScale = new Vector3(0.12f, 0.35f * def.limbLengthScale * scale, 0.12f);
            limb.localPosition = new Vector3(x, 1.05f * def.heightScale * scale, 0f);
        }

        private static void BuildFoot(
            Transform parent, string name, float x, float bodyWidth, CharacterDefinition def, float scale)
        {
            var foot = CreatePart(PrimitiveType.Cube, name, parent);
            foot.localScale = new Vector3(bodyWidth * 0.35f, 0.12f, 0.45f);
            foot.localPosition = new Vector3(x, 0.35f * def.heightScale * scale, 0f);
        }

        private static void ApplyMaterial(GameObject root, CharacterDefinition definition)
        {
            var material = FactionMaterialFactory.Create(definition);
            foreach (var renderer in root.GetComponentsInChildren<Renderer>())
            {
                renderer.sharedMaterial = material;
            }
        }
    }
}
