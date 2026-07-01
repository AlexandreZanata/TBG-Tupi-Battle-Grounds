using NUnit.Framework;
using TBG.CharacterLibrary.Locomotion;
using UnityEngine;

namespace TBG.CharacterLibrary.Tests
{
    public sealed class TwoBoneIkTests
    {
        [Test]
        public void ClampReach_ShortensTargetsBeyondMaxLength()
        {
            var root = Vector3.zero;
            var target = new Vector3(0f, 0f, 5f);
            var clamped = TwoBoneIk.ClampReach(root, target, 2f);
            Assert.That(clamped.magnitude, Is.EqualTo(2f).Within(1e-4f));
        }

        [Test]
        public void Solve_ReachesTargetWithinChainLength()
        {
            var rootGo = new GameObject("IkRoot");
            var upperGo = new GameObject("Upper");
            var lowerGo = new GameObject("Lower");
            var endGo = new GameObject("End");
            upperGo.transform.SetParent(rootGo.transform);
            lowerGo.transform.SetParent(upperGo.transform);
            endGo.transform.SetParent(lowerGo.transform);
            upperGo.transform.localPosition = new Vector3(0f, 0.4f, 0f);
            lowerGo.transform.localPosition = new Vector3(0f, 0.4f, 0f);
            endGo.transform.localPosition = new Vector3(0f, 0.2f, 0f);

            var target = rootGo.transform.position + new Vector3(0.1f, 0.2f, 0.35f);
            var solved = TwoBoneIk.Solve(
                upperGo.transform,
                lowerGo.transform,
                endGo.transform,
                target,
                rootGo.transform.position + Vector3.right);

            Assert.That(solved, Is.True);
            Assert.That(Vector3.Distance(endGo.transform.position, target), Is.LessThan(0.25f));
            Object.DestroyImmediate(rootGo);
        }
    }
}
