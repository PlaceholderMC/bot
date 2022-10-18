import type { CacheType, ChatInputCommandInteraction } from 'discord.js';
import { COLORS } from '../constants';

export const starsCommand = async (
  i: ChatInputCommandInteraction<CacheType>
) => {
  await i.deferReply();

  /* CHANGEME */
  const count = await fetch('https://api.github.com/repos/PrismLauncher/PrismLauncher')
    .then((r) => r.json() as Promise<{ stargazers_count: number }>)
    .then((j) => j.stargazers_count);

  await i.editReply({
    embeds: [
      {
        title: `⭐ ${count} total stars!`,
        color: COLORS.yellow,
      },
    ],
  });
};
