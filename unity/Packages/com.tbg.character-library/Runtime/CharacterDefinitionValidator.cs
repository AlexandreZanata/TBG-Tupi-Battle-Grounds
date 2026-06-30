using System;
using System.Collections.Generic;

namespace TBG.CharacterLibrary
{
    public sealed class CharacterDefinitionValidationException : Exception
    {
        public CharacterDefinitionValidationException(IReadOnlyList<string> errors)
            : base(string.Join("; ", errors))
        {
            Errors = errors;
        }

        public IReadOnlyList<string> Errors { get; }
    }

    public static class CharacterDefinitionValidator
    {
        private const float ProportionMin = 0.1f;
        private const float ProportionMax = 3.0f;

        public static IReadOnlyList<string> Validate(CharacterDefinitionJsonRecord record)
        {
            var errors = new List<string>();
            if (record == null || string.IsNullOrWhiteSpace(record.id))
            {
                errors.Add("id must be non-empty");
                return errors;
            }

            if (record.id.StartsWith("_", StringComparison.Ordinal))
            {
                errors.Add("id must not be a schema-only file");
            }

            if (!IsValidAssetMode(record.assetMode))
            {
                errors.Add($"invalid assetMode: {record.assetMode}");
            }

            ValidateProportions(record, errors);
            ValidateMesh3dFields(record, errors);
            ValidateTextureFields(record, errors);
            ValidateOptionalProfiles(record, errors);
            return errors;
        }

        public static void ValidateOrThrow(CharacterDefinitionJsonRecord record)
        {
            var errors = Validate(record);
            if (errors.Count > 0)
            {
                throw new CharacterDefinitionValidationException(errors);
            }
        }

        private static bool IsValidAssetMode(string value)
        {
            if (string.IsNullOrWhiteSpace(value))
            {
                return true;
            }

            return string.Equals(value, "procedural", StringComparison.OrdinalIgnoreCase)
                || string.Equals(value, "mesh3d", StringComparison.OrdinalIgnoreCase);
        }

        private static void ValidateProportions(CharacterDefinitionJsonRecord record, List<string> errors)
        {
            ValidateProportion(record.heightScale, "heightScale", errors);
            ValidateProportion(record.bodyWidthScale, "bodyWidthScale", errors);
            ValidateProportion(record.headScale, "headScale", errors);
            ValidateProportion(record.limbLengthScale, "limbLengthScale", errors);
        }

        private static void ValidateProportion(float value, string fieldName, List<string> errors)
        {
            if (value < ProportionMin || value > ProportionMax)
            {
                errors.Add($"{fieldName} out of range ({ProportionMin}–{ProportionMax})");
            }
        }

        private static void ValidateMesh3dFields(CharacterDefinitionJsonRecord record, List<string> errors)
        {
            if (!string.Equals(record.assetMode, "mesh3d", StringComparison.OrdinalIgnoreCase))
            {
                return;
            }

            if (string.IsNullOrWhiteSpace(record.mesh))
            {
                errors.Add("mesh3d assetMode requires mesh path");
                return;
            }

            if (!record.mesh.EndsWith(".glb", StringComparison.OrdinalIgnoreCase))
            {
                errors.Add("mesh path must be a .glb file");
            }
        }

        private static void ValidateTextureFields(CharacterDefinitionJsonRecord record, List<string> errors)
        {
            if (record.textures == null)
            {
                return;
            }

            RequireTexturePath(record.textures.albedo, "textures.albedo", errors);
            RequireTexturePath(record.textures.normal, "textures.normal", errors);
            RequireTexturePath(record.textures.orm, "textures.orm", errors);
            if (record.textures.emissive != null && string.IsNullOrWhiteSpace(record.textures.emissive))
            {
                errors.Add("textures.emissive must be non-empty when provided");
            }
        }

        private static void ValidateOptionalProfiles(CharacterDefinitionJsonRecord record, List<string> errors)
        {
            RequireNonEmptyWhenPresent(record.materialProfile, "materialProfile", errors);
            RequireNonEmptyWhenPresent(record.behaviorProfile, "behaviorProfile", errors);
            RequireNonEmptyWhenPresent(record.animatorController, "animatorController", errors);
        }

        private static void RequireTexturePath(string value, string fieldName, List<string> errors)
        {
            if (string.IsNullOrWhiteSpace(value))
            {
                errors.Add($"{fieldName} is required when textures object is present");
            }
        }

        private static void RequireNonEmptyWhenPresent(string value, string fieldName, List<string> errors)
        {
            if (value != null && string.IsNullOrWhiteSpace(value))
            {
                errors.Add($"{fieldName} must be non-empty when provided");
            }
        }
    }
}
