export const Poem: React.VFC<{ seed: string; sentence: string[] }> = ({
  seed,
  sentence,
}) => {
  const charPositions: { [key: string]: number[] } = {};

  seed.split("").forEach((letter, index) => {
    if (letter in charPositions) {
      charPositions[letter].push(index);
    } else {
      charPositions[letter] = [index];
    }
  });

  return (
    <div className="flex">
      {sentence.map((word, index) => (
        <div className={`flex ${index > 0 && "ml-4"}`} key={index}>
          {word.split("").map((letter, index) => {
            const letterIndex = charPositions[letter].shift()!;
            const offset = seed.length - letterIndex;
            const filler = new Array(offset).fill("x");
            return (
              <div className="flex flex-col" key={index}>
                {filler.map((x, index) => {
                  return (
                    <div key={index} className="leading-5">
                      &nbsp;
                    </div>
                  );
                })}
                {seed.split("").map((letter, index) => {
                  return (
                    <div
                      key={index}
                      className={`w-6 leading-5 text-center ${
                        index !== letterIndex && "text-gray-300"
                      }`}
                    >
                      {letter === " " ? <span>&nbsp;</span> : letter}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
};
