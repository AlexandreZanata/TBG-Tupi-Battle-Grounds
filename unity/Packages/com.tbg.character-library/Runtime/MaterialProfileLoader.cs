using System;
using System.Collections.Generic;
using System.IO;
using TBG.CharacterSystem.Runtime;
using UnityEngine;

namespace TBG.CharacterLibrary
{
    public sealed class MaterialProfileLoader
    {
        private static readonly string[] RequiredSlots = { "albedo", "normal", "orm" };

        public MaterialProfile Load(string profileId)
        {
            if (string.IsNullOrWhiteSpace(profileId))
            {
                throw new ArgumentException("Material profile id is required.", nameof(profileId));
            }

            var path = Path.Combine(CharacterPaths.MaterialsDir, profileId + ".json");
            if (!File.Exists(path))
            {
                throw new FileNotFoundException("Material profile not found.", path);
            }

            var json = File.ReadAllText(path);
            var record = JsonUtility.FromJson<MaterialProfileJsonRecord>(json);
            ValidateOrThrow(record);
            return MaterialProfile.FromRecord(record);
        }

        public Material CreateMaterial(MaterialProfile profile)
        {
            if (profile == null)
            {
                throw new ArgumentNullException(nameof(profile));
            }

            var shader = Shader.Find(profile.ShaderName);
            if (shader == null)
            {
                throw new InvalidOperationException($"Shader not found: {profile.ShaderName}");
            }

            var material = new Material(shader)
            {
                name = profile.Id
            };
            material.SetFloat("_Metallic", profile.Metallic);
            material.SetFloat("_Smoothness", profile.Smoothness);
            material.SetFloat("_OcclusionStrength", profile.OcclusionStrength);
            material.SetFloat("_EmissionStrength", profile.EmissionStrength);
            return material;
        }

        public static void ValidateOrThrow(MaterialProfileJsonRecord record)
        {
            var errors = Validate(record);
            if (errors.Count > 0)
            {
                throw new InvalidOperationException(string.Join("; ", errors));
            }
        }

        public static List<string> Validate(MaterialProfileJsonRecord record)
        {
            var errors = new List<string>();
            if (record == null || string.IsNullOrWhiteSpace(record.id))
            {
                errors.Add("material profile id is required");
                return errors;
            }

            if (string.IsNullOrWhiteSpace(record.shader))
            {
                errors.Add("shader is required");
            }

            if (record.textureSlots == null || record.textureSlots.Length == 0)
            {
                errors.Add("textureSlots must not be empty");
                return errors;
            }

            foreach (var slot in RequiredSlots)
            {
                if (!Array.Exists(record.textureSlots, s => s == slot))
                {
                    errors.Add($"textureSlots must include {slot}");
                }
            }

            return errors;
        }
    }
}
