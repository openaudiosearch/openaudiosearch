const path = require('path')

const HtmlWebpackPlugin = require('html-webpack-plugin')
const ReactRefreshPlugin = require('@pmmmwh/react-refresh-webpack-plugin')

const getPath = (file) => {
  return path.resolve(__dirname, 'src', file)
}

module.exports = (env, argv) => {
  const isDevelopment = process.env.NODE_ENV === 'development'
  const filename = isDevelopment ? '[name]' : '[name]-[contenthash:6]'

  const config = {
    entry: {
      app: getPath('index.js')
    },
    mode: isDevelopment ? 'development' : 'production',
    output: {
      filename: `${filename}.js`,
      sourceMapFilename: `${filename}.js.map`
    },
    resolve: {
      extensions: ['.js', '.jsx']
    },
    devtool: 'source-map',
    module: {
      rules: [
        {
          test: /\.(t|j)sx?/,
          exclude: /node_modules/,
          use: [
            {
              loader: 'babel-loader'
            }
          ]
        },
        {
          test: /\.css$/i,
          use: [
            'style-loader',
            'css-loader'
          ]
        },
        {
          test: /\.(woff|woff2|eot|ttf|otf)$/,
          use: [
            'file-loader'
          ]
        }
      ]
    },
    plugins: [
      isDevelopment && new ReactRefreshPlugin(),
      new HtmlWebpackPlugin({
        filename: 'index.html',
        template: getPath('index.html')
      })
    ].filter(Boolean)
  }
  if (isDevelopment) {
    config.devServer = {
      port: 4000
    }
  }
  return config
}
