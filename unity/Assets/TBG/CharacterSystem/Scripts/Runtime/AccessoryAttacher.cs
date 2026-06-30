using TBG.CharacterSystem.Domain;
using UnityEngine;

namespace TBG.CharacterSystem.Runtime
{
    public class AccessoryAttacher
    {
        public void Attach(Transform root, AccessoryType[] accessories, float bodyWidth)
        {
            if (accessories == null)
            {
                return;
            }

            foreach (var accessory in accessories)
            {
                AttachSingle(root, accessory, bodyWidth);
            }
        }

        private static void AttachSingle(Transform root, AccessoryType accessory, float bodyWidth)
        {
            var part = GameObject.CreatePrimitive(PrimitiveType.Cube);
            part.name = accessory.ToString();
            part.transform.SetParent(root, false);

            switch (accessory)
            {
                case AccessoryType.Shield:
                    part.transform.localPosition = new Vector3(-bodyWidth * 0.7f, 1.1f, 0f);
                    part.transform.localScale = new Vector3(0.15f, 0.5f, 0.05f);
                    break;
                case AccessoryType.Weapon:
                    part.transform.localPosition = new Vector3(bodyWidth * 0.6f, 1.2f, 0f);
                    part.transform.localScale = new Vector3(0.08f, 0.7f, 0.08f);
                    break;
                case AccessoryType.Helmet:
                    part.transform.localPosition = new Vector3(0f, 2.05f, 0f);
                    part.transform.localScale = new Vector3(0.45f, 0.12f, 0.45f);
                    break;
                case AccessoryType.ArmorBlock:
                    part.transform.localPosition = new Vector3(0f, 1.15f, 0.12f);
                    part.transform.localScale = new Vector3(bodyWidth * 0.8f, 0.35f, 0.12f);
                    break;
            }

            Object.Destroy(part.GetComponent<Collider>());
        }
    }
}
