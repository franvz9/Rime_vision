import SwiftUI

struct DependencyGraphView: View {
    let schemas: [RimeSchema]
    @State private var selectedSchema: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("方案依赖关系")
                .font(.headline)

            ScrollView {
                VStack(alignment: .leading, spacing: 8) {
                    ForEach(schemas) { schema in
                        SchemaNodeView(
                            schema: schema,
                            allSchemas: schemas,
                            isSelected: selectedSchema == schema.schemaId,
                            onSelect: { selectedSchema = schema.schemaId }
                        )
                    }
                }
                .padding()
            }
        }
        .padding()
    }
}

struct SchemaNodeView: View {
    let schema: RimeSchema
    let allSchemas: [RimeSchema]
    let isSelected: Bool
    let onSelect: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Circle()
                    .fill(schema.enabled ? Color.green : Color.gray)
                    .frame(width: 8, height: 8)

                Text(schema.schemaId)
                    .font(.system(.body, design: .monospaced))
                    .fontWeight(isSelected ? .semibold : .regular)

                if !schema.name.isEmpty {
                    Text("- \(schema.name)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                if !schema.dependencies.isEmpty {
                    Text("\(schema.dependencies.count) 依赖")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Color.secondary.opacity(0.1))
                        .cornerRadius(4)
                }
            }

            if !schema.dependencies.isEmpty {
                VStack(alignment: .leading, spacing: 2) {
                    ForEach(schema.dependencies, id: \.self) { depId in
                        HStack(spacing: 4) {
                            Text("  →")
                                .font(.caption)
                                .foregroundColor(.accentColor)
                            Text(depId)
                                .font(.system(.caption, design: .monospaced))
                                .foregroundColor(.secondary)

                            if !allSchemas.contains(where: { $0.schemaId == depId }) {
                                Text("(未安装)")
                                    .font(.caption2)
                                    .foregroundColor(.orange)
                            }
                        }
                    }
                }
                .padding(.leading, 20)
            }
        }
        .padding(8)
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(isSelected ? Color.accentColor.opacity(0.1) : Color.clear)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .stroke(isSelected ? Color.accentColor : Color.clear, lineWidth: 1)
        )
        .onTapGesture { onSelect() }
    }
}
