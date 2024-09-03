import formsPlugin from '@tailwindcss/forms'
import typographyPlugin from '@tailwindcss/typography'
import type { Config } from 'tailwindcss'
import colors from 'tailwindcss/colors'
import plugin from 'tailwindcss/plugin'

const accentColorBucket = colors.blue
const neutralColorBucket = colors.stone
const borderColor = neutralColorBucket[200]

const innerRingStartOpacity = 0.8
const innerRingEndOpacity = 0.5
const outerRingStartOpacity = 0.5
const outerRingEndOpacity = 0.1

const config: Config = {
  content: [
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  darkMode: 'class',
  theme: {
    extend: {
      borderColor: {
        DEFAULT: borderColor,
      },
      outlineColor: {
        DEFAULT: borderColor,
      },
      colors: {
        accentColor: accentColorBucket[500],
        appBgColor: neutralColorBucket[50],
        borderColor,
        neutral: neutralColorBucket,
        textColor: neutralColorBucket[800],
        fadedTextColor: neutralColorBucket[500],
        shadedBgColor: accentColorBucket[100],
      },
      fontFamily: {
        body: ['var(--font-raleway)', 'sans-serif'],
        display: ['var(--font-bitter)', 'sans-serif'],
        icon: ["'Font Awesome 6 Pro'"],
      },
      animation: {
        animateInX: 'animateInX 0.5s ease-in-out both',
        animateInY: 'animateInY 0.5s ease-in-out both',
        animateOutX: 'animateOutX 0.5s ease-in-out both',
        animateOutY: 'animateOutY 0.5s ease-in-out both',
        float: 'float 2.5s ease-in-out infinite alternate',
        floatUp: 'floatUp 120s linear infinite',
        forceFieldSmallInner:
          'forceFieldSmallInner 2s ease-in-out infinite alternate',
        forceFieldSmallOuter:
          'forceFieldSmallOuter 2s ease-in-out infinite alternate',
        forceFieldInner: 'forceFieldInner 2s ease-in-out infinite alternate',
        forceFieldOuter: 'forceFieldOuter 2s ease-in-out infinite alternate',
        rotatingSphere: 'rotatingSphere 120s linear infinite',
        slowDriftLeft: 'driftLeft 60s linear infinite',
        slowerDriftLeft: 'driftLeft 120s linear infinite',
        slowestDriftLeft: 'driftLeft 180s linear infinite',
        slowTwinkle: 'twinkle 0.25s linear infinite alternate',
        slowerTwinkle: 'twinkle 0.5s linear infinite alternate',
        wobble: 'wobble 2s ease-in-out infinite alternate',
      },
      keyframes: {
        animateInX: {
          '0%': {
            opacity: '0',
            transform: 'translateX(100%)',
          },
          '100%': {
            opacity: '1',
            transform: 'translateX(0)',
          },
        },
        animateOutX: {
          '0%': {
            maxHeight: '200px',
            opacity: '1',
            transform: 'translateX(0%)',
          },
          '70%': {
            maxHeight: '200px',
            opacity: '0',
            transform: 'translateX(100%)',
          },
          '100%': {
            maxHeight: '0',
            opacity: '0',
            transform: 'translateX(100%)',
          },
        },

        animateInY: {
          '0%': {
            opacity: '0',
            transform: 'translateY(100%)',
          },
          '100%': {
            opacity: '1',
            transform: 'translateY(0)',
          },
        },
        animateOutY: {
          '0%': {
            opacity: '1',
            transform: 'translateY(0%)',
          },
          '100%': {
            opacity: '0',
            transform: 'translateY(-100%)',
          },
        },

        driftLeft: {
          '0%': {
            opacity: '0',
            transform: 'translateX(0)',
          },
          '5%, 95%': {
            opacity: '1',
          },
          '100%': {
            opacity: '0',
            transform: 'translateX(-120vw)',
          },
        },

        float: {
          '0%': {
            transform: 'translateY(-3%)',
          },
          '100%': {
            transform: 'translateY(3%)',
          },
        },

        floatUp: {
          '0%': {
            transform: 'translateX(0) translateY(0)',
          },
          '100%': {
            transform: 'translateX(10vw) translateY(-110vh) rotate(10deg)',
          },
        },

        forceFieldSmallInner: {
          '0%': {
            opacity: String(innerRingStartOpacity),
            strokeWidth: '5px',
          },
          '100%': {
            opacity: String(innerRingEndOpacity),
            strokeWidth: '20px',
          },
        },

        forceFieldSmallOuter: {
          '0%': {
            opacity: String(outerRingStartOpacity),
            strokeWidth: '10px',
          },
          '100%': {
            opacity: String(outerRingEndOpacity),
            strokeWidth: '30px',
          },
        },

        forceFieldInner: {
          '0%': {
            opacity: String(innerRingStartOpacity),
            strokeWidth: '10px',
          },
          '100%': {
            opacity: String(innerRingEndOpacity),
            strokeWidth: '60px',
          },
        },

        forceFieldOuter: {
          '0%': {
            opacity: String(outerRingStartOpacity),
            strokeWidth: '20px',
          },
          '100%': {
            opacity: String(outerRingEndOpacity),
            strokeWidth: '100px',
          },
        },

        rotatingSphere: {
          '0%': {
            transform: 'rotate(0)',
          },
          '100%': {
            transform: 'rotate(359deg)',
          },
        },

        twinkle: {
          '0%': {
            opacity: '0.9',
          },
          '50%': {
            opacity: '0.8',
          },
          '100%': {
            opacity: '1',
          },
        },

        wobble: {
          '0%': {
            transform: 'rotate(-3deg)',
          },
          '100%': {
            transform: 'rotate(3deg)',
          },
        },
      },
    },
  },
  plugins: [
    plugin(function ({ addBase, theme }) {
      addBase({
        '*': {
          scrollbarColor: `${theme('colors.accentColor')} transparent`,
        },
        '*::-webkit-scrollbar': {
          height: theme('spacing.2'),
          width: theme('spacing.2'),
        },
        '*::-webkit-scrollbar-track': {
          background: 'transparent',
        },
        '*::-webkit-scrollbar-thumb': {
          background: theme('colors.accentColor'),
          borderRadius: theme('spacing.8'),
        },
        'a, button, input, textarea': {
          touchAction: 'manipulation',
        },
      })
    }),
    formsPlugin,
    typographyPlugin,
  ],
}
export default config
