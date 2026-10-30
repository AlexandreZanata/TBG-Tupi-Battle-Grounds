import { connect } from './tbg-a1-cdp.mjs';

async function main() {
  const cdp = await connect(9333);
  console.log('Connected to Chrome!');
  
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery=aim&unit=archer&angle=0' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/archer-aim-angle-0.jpg');
  console.log('Saved archer-aim-angle-0.jpg');

  for (let angle = 1; angle < 8; angle++) {
    await cdp.send('Page.navigate', { url: `http://127.0.0.1:8131/?gallery=aim&unit=archer&angle=${angle}` });
    await new Promise(r => setTimeout(r, 1200));
    await cdp.screenshot(`docs/media/a1/archer-aim-angle-${angle}.jpg`);
    console.log(`Saved archer-aim-angle-${angle}.jpg`);
  }

  // Also capture Shieldbearer and Bone Brute in melee aim lab
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery=aim&unit=shieldbearer&angle=2' });
  await new Promise(r => setTimeout(r, 1500));
  await cdp.screenshot('docs/media/a1/melee-shieldbearer-aim.jpg');
  console.log('Saved melee-shieldbearer-aim.jpg');

  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery=aim&unit=bonebrute&angle=6' });
  await new Promise(r => setTimeout(r, 1500));
  await cdp.screenshot('docs/media/a1/melee-bonebrute-aim.jpg');
  console.log('Saved melee-bonebrute-aim.jpg');

  // Crowd mode (12 attackers around 3 targets)
  await cdp.send('Page.navigate', { url: 'http://127.0.0.1:8131/?gallery=aim&unit=shieldbearer&crowd' });
  await new Promise(r => setTimeout(r, 2000));
  await cdp.screenshot('docs/media/a1/melee-crowd-aim.jpg');
  console.log('Saved melee-crowd-aim.jpg');

  cdp.close();
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
